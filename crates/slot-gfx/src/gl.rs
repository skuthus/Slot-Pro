use crate::surface::{GfxError, Surface};
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};

static ES: AtomicBool = AtomicBool::new(false);

pub fn load(surface: &dyn Surface) {
    gl::load_with(|name| surface.proc_address(name));
    let version = unsafe { gl::GetString(gl::VERSION) };
    let es = !version.is_null()
        && unsafe { std::ffi::CStr::from_ptr(version as *const std::ffi::c_char) }
            .to_string_lossy()
            .starts_with("OpenGL ES");
    ES.store(es, Ordering::Relaxed);
}

pub fn es() -> bool {
    ES.load(Ordering::Relaxed)
}

const LUMINANCE: gl::types::GLenum = 0x1909;

pub fn gray_format() -> gl::types::GLenum {
    match es() {
        true => LUMINANCE,
        false => gl::RED,
    }
}

pub fn internal_format(format: gl::types::GLenum) -> gl::types::GLint {
    match (es(), format) {
        (true, _) => format as gl::types::GLint,
        (false, gl::RED) => gl::R8 as gl::types::GLint,
        (false, _) => gl::RGBA8 as gl::types::GLint,
    }
}

fn bytes_per_pixel(format: gl::types::GLenum) -> u32 {
    match format {
        gl::RED | LUMINANCE => 1,
        _ => 4,
    }
}

fn shader(kind: gl::types::GLenum, src: &str) -> Result<gl::types::GLuint, GfxError> {
    let Ok(csrc) = CString::new(src) else {
        return Err(GfxError::Shader("source contains a nul byte".into()));
    };
    unsafe {
        let id = gl::CreateShader(kind);
        gl::ShaderSource(id, 1, &csrc.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
        let mut ok = 0;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut ok);
        if ok == 0 {
            let log = info_log(id, gl::GetShaderiv, gl::GetShaderInfoLog);
            gl::DeleteShader(id);
            return Err(GfxError::Shader(log));
        }
        Ok(id)
    }
}

static SHADER_CACHE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

const NUM_PROGRAM_BINARY_FORMATS: gl::types::GLenum = 0x87FE;
const PROGRAM_BINARY_LENGTH: gl::types::GLenum = 0x8741;

pub fn set_shader_cache(dir: std::path::PathBuf) {
    let _ = SHADER_CACHE.set(dir);
}

pub fn program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let cached = cache_path(vert, frag);
    if let Some(p) = cached.as_deref().and_then(load_binary) {
        return Ok(p);
    }
    let p = compile_program(vert, frag)?;
    if let Some(path) = cached {
        store_binary(&path, p);
    }
    Ok(p)
}

fn cache_path(vert: &str, frag: &str) -> Option<std::path::PathBuf> {
    let dir = SHADER_CACHE.get()?;
    if !gl::ProgramBinary::is_loaded() || !gl::GetProgramBinary::is_loaded() {
        return None;
    }
    let mut formats = 0;
    unsafe { gl::GetIntegerv(NUM_PROGRAM_BINARY_FORMATS, &mut formats) };
    if formats <= 0 {
        return None;
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let driver = [gl::RENDERER, gl::VERSION].map(|name| unsafe {
        let s = gl::GetString(name);
        match s.is_null() {
            true => Vec::new(),
            false => std::ffi::CStr::from_ptr(s as *const std::ffi::c_char)
                .to_bytes()
                .to_vec(),
        }
    });
    for part in [
        &driver[0][..],
        &driver[1][..],
        vert.as_bytes(),
        frag.as_bytes(),
    ] {
        for &b in part.iter().chain(&[0]) {
            hash = (hash ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
    }
    Some(dir.join(format!("{hash:016x}.bin")))
}

fn load_binary(path: &std::path::Path) -> Option<gl::types::GLuint> {
    let bytes = std::fs::read(path).ok()?;
    let (format, binary) = bytes.split_first_chunk::<4>()?;
    unsafe {
        let p = gl::CreateProgram();
        gl::ProgramBinary(
            p,
            u32::from_le_bytes(*format),
            binary.as_ptr() as *const std::ffi::c_void,
            binary.len() as gl::types::GLsizei,
        );
        let mut ok = 0;
        gl::GetProgramiv(p, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            gl::DeleteProgram(p);
            let _ = std::fs::remove_file(path);
            eprintln!(
                "slot: gfx: dropped a stale shader binary {}",
                path.display()
            );
            return None;
        }
        eprintln!("slot: gfx: shader from {}", path.display());
        Some(p)
    }
}

fn store_binary(path: &std::path::Path, p: gl::types::GLuint) {
    unsafe {
        let mut len = 0;
        gl::GetProgramiv(p, PROGRAM_BINARY_LENGTH, &mut len);
        if len <= 0 {
            return;
        }
        let mut binary = vec![0u8; len as usize];
        let (mut written, mut format) = (0, 0);
        gl::GetProgramBinary(
            p,
            len,
            &mut written,
            &mut format,
            binary.as_mut_ptr() as *mut std::ffi::c_void,
        );
        if written <= 0 {
            return;
        }
        binary.truncate(written as usize);
        let mut bytes = format.to_le_bytes().to_vec();
        bytes.extend(binary);
        let stored = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(path, bytes));
        match stored {
            Ok(()) => eprintln!("slot: gfx: kept shader {}", path.display()),
            Err(e) => eprintln!("slot: gfx: could not keep a shader binary: {e}"),
        }
    }
}

fn compile_program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let vs = shader(gl::VERTEX_SHADER, vert)?;
    let fs = match shader(gl::FRAGMENT_SHADER, frag) {
        Ok(fs) => fs,
        Err(e) => {
            unsafe { gl::DeleteShader(vs) };
            return Err(e);
        }
    };
    unsafe {
        let p = gl::CreateProgram();
        gl::AttachShader(p, vs);
        gl::AttachShader(p, fs);
        if let Ok(pos) = CString::new("a_pos") {
            gl::BindAttribLocation(p, crate::quad::POS_LOCATION, pos.as_ptr());
        }
        gl::LinkProgram(p);
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);
        let mut ok = 0;
        gl::GetProgramiv(p, gl::LINK_STATUS, &mut ok);
        if ok == 0 {
            let log = info_log(p, gl::GetProgramiv, gl::GetProgramInfoLog);
            gl::DeleteProgram(p);
            return Err(GfxError::Shader(log));
        }
        Ok(p)
    }
}

unsafe fn info_log(
    id: gl::types::GLuint,
    get_iv: unsafe fn(gl::types::GLuint, gl::types::GLenum, *mut gl::types::GLint),
    get_log: unsafe fn(
        gl::types::GLuint,
        gl::types::GLsizei,
        *mut gl::types::GLsizei,
        *mut gl::types::GLchar,
    ),
) -> String {
    let mut len = 0;
    get_iv(id, gl::INFO_LOG_LENGTH, &mut len);
    if len <= 0 {
        return "no info log".into();
    }
    let mut buf = vec![0u8; len as usize];
    get_log(
        id,
        len,
        std::ptr::null_mut(),
        buf.as_mut_ptr() as *mut gl::types::GLchar,
    );
    while buf.last() == Some(&0) {
        buf.pop();
    }
    String::from_utf8_lossy(&buf).into_owned()
}

pub fn texture(
    w: u32,
    h: u32,
    filter: gl::types::GLenum,
    wrap: gl::types::GLenum,
    format: gl::types::GLenum,
    data: Option<&[u8]>,
) -> gl::types::GLuint {
    let pixels = match data {
        Some(d) if d.len() >= (w * h * bytes_per_pixel(format)) as usize => {
            d.as_ptr() as *const std::ffi::c_void
        }
        _ => std::ptr::null(),
    };
    unsafe {
        let mut tex = 0;
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal_format(format),
            w as i32,
            h as i32,
            0,
            format,
            gl::UNSIGNED_BYTE,
            pixels,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as i32);
        tex
    }
}

pub fn uniform_location(program: gl::types::GLuint, name: &str) -> gl::types::GLint {
    match CString::new(name) {
        Ok(c) => unsafe { gl::GetUniformLocation(program, c.as_ptr()) },
        Err(_) => -1,
    }
}
