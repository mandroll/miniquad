use crate::{sapp::*, Context};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texture {
    pub(crate) texture: GLuint,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn empty() -> Texture {
        Texture {
            texture: 0,
            width: 0,
            height: 0,
        }
    }

    /// Delete GPU texture, leaving handle unmodified.
    ///
    /// More high-level code on top of miniquad probably is going to call this in Drop implementation of some
    /// more RAII buffer object.
    ///
    /// There is no protection against using deleted textures later. However its not an UB in OpenGl and thats why
    /// this function is not marked as unsafe
    pub fn delete(&self) {
        unsafe {
            glDeleteTextures(1, &self.texture as *const _);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderTextureFormat {
    RGBA8,
    Depth,
}

impl From<RenderTextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: RenderTextureFormat) -> Self {
        match format {
            RenderTextureFormat::RGBA8 => (GL_RGBA, GL_RGBA, GL_UNSIGNED_BYTE),
            RenderTextureFormat::Depth => {
                (GL_DEPTH_COMPONENT, GL_DEPTH_COMPONENT, GL_UNSIGNED_SHORT)
            }
        }
    }
}

/// List of all the possible formats of input data when uploading to texture.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    RGB8,
    RGBA8,
    RGBA4,
    RGBA5551,
    RGB565,
    ALPHA,
}

impl From<TextureFormat> for (GLenum, GLenum, GLenum) {
    fn from(format: TextureFormat) -> Self {
        match format {
            TextureFormat::ALPHA => (GL_ALPHA, GL_ALPHA, GL_UNSIGNED_BYTE),
            TextureFormat::RGB8 => (GL_RGB8, GL_RGB, GL_UNSIGNED_BYTE),
            TextureFormat::RGBA8 => (GL_RGBA8, GL_RGBA, GL_UNSIGNED_BYTE),
            TextureFormat::RGB565 => (GL_RGB565, GL_RGB, GL_UNSIGNED_SHORT_5_6_5),
            TextureFormat::RGBA4 => (GL_RGBA4, GL_RGBA, GL_UNSIGNED_SHORT_4_4_4_4),
            TextureFormat::RGBA5551 => (GL_RGB5_A1, GL_RGBA, GL_UNSIGNED_SHORT_5_5_5_1),
        }
    }
}

impl TextureFormat {
    /// Returns the size in bytes of texture with `dimensions`.
    pub fn size(self, width: u32, height: u32) -> u32 {
        let square = width * height;

        match self {
            TextureFormat::ALPHA => square,
            TextureFormat::RGB565 | TextureFormat::RGBA4 | TextureFormat::RGBA5551 => 2 * square,
            TextureFormat::RGB8 => 3 * square,
            TextureFormat::RGBA8 => 4 * square,
        }
    }
}

impl Default for TextureParams {
    fn default() -> Self {
        TextureParams {
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            filter: FilterMode::Linear,
            width: 0,
            height: 0,
        }
    }
}

/// Sets the wrap parameter for texture.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextureWrap {
    /// Samples at coord x + 1 map to coord x.
    Repeat,
    /// Samples at coord x + 1 map to coord 1 - x.
    Mirror,
    /// Samples at coord x + 1 map to coord 1.
    Clamp,
    /// Same as Mirror, but only for one repetition.
    MirrorClamp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    Linear = GL_LINEAR as isize,
    Nearest = GL_NEAREST as isize,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureParams {
    pub format: TextureFormat,
    pub wrap: TextureWrap,
    pub filter: FilterMode,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct RenderTextureParams {
    pub format: RenderTextureFormat,
    pub wrap: TextureWrap,
    pub filter: FilterMode,
    pub width: u32,
    pub height: u32,
}

impl Default for RenderTextureParams {
    fn default() -> Self {
        RenderTextureParams {
            format: RenderTextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            filter: FilterMode::Linear,
            width: 0,
            height: 0,
        }
    }
}

impl Texture {
    pub fn new_render_texture(ctx: &mut Context, params: RenderTextureParams) -> Texture {
        let (internal_format, format, pixel_type) = params.format.into();

        ctx.cache.store_texture_binding(0);

        let mut texture: GLuint = 0;

        unsafe {
            glGenTextures(1, &mut texture as *mut _);
            glActiveTexture(GL_TEXTURE0);
            ctx.cache.bind_texture(0, texture);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                internal_format as i32,
                params.width as i32,
                params.height as i32,
                0,
                format,
                pixel_type,
                std::ptr::null(),
            );

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);
        }
        ctx.cache.restore_texture_binding(0);

        Texture {
            texture,
            width: params.width,
            height: params.height,
        }
    }

    /// Upload texture to GPU with given TextureParams
    pub fn from_data_and_format(ctx: &mut Context, bytes: &[u8], params: TextureParams) -> Texture {
        assert_eq!(params.format.size(params.width, params.height), bytes.len() as u32);

        let (internal_format, format, pixel_type) = params.format.into();

        unsafe {
            ctx.cache.store_texture_binding(0);

            let mut texture: GLuint = 0;
            glGenTextures(1, &mut texture as *mut _);
            ctx.cache.bind_texture(0, texture);
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                internal_format as i32,
                params.width as i32,
                params.height as i32,
                0,
                format,
                pixel_type,
                bytes.as_ptr() as *const _,
            );

            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

            ctx.cache.restore_texture_binding(0);

            Texture {
                texture,
                width: params.width as u32,
                height: params.height as u32,
            }
        }
    }

    /// Upload RGBA8 texture to GPU
    pub fn from_rgba8(ctx: &mut Context, width: u16, height: u16, bytes: &[u8]) -> Texture {
        assert_eq!(width as usize * height as usize * 4, bytes.len());

        Self::from_data_and_format(
            ctx,
            bytes,
            TextureParams {
                width: width as _,
                height: height as _,
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Linear,
            },
        )
    }

    pub fn set_filter(&self, ctx: &mut Context, filter: FilterMode) {
        ctx.cache.store_texture_binding(0);
        ctx.cache.bind_texture(0, self.texture);
        unsafe {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter as i32);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter as i32);
        }
        ctx.cache.restore_texture_binding(0);
    }

    /// Update whole texture content
    /// bytes should be width * height * 4 size - non rgba8 textures are not supported yet anyway
    pub fn update(&self, ctx: &mut Context, bytes: &[u8]) {
        assert_eq!(self.width as usize * self.height as usize * 4, bytes.len());

        self.update_texture_part(
            ctx,
            0 as _,
            0 as _,
            self.width as _,
            self.height as _,
            bytes,
        )
    }

    pub fn update_texture_part(
        &self,
        ctx: &mut Context,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        bytes: &[u8],
    ) {
        assert_eq!(width as usize * height as usize * 4, bytes.len());
        assert!(x_offset + width <= self.width as _);
        assert!(y_offset + height <= self.height as _);

        ctx.cache.store_texture_binding(0);
        ctx.cache.bind_texture(0, self.texture);

        unsafe {
            glTexSubImage2D(
                GL_TEXTURE_2D,
                0,
                x_offset as _,
                y_offset as _,
                width as _,
                height as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                bytes.as_ptr() as *const _,
            );
        }

        ctx.cache.restore_texture_binding(0);
    }
}