use glam::UVec2;
use paint_core::persistence::{Error, Texture, TextureFormat};
use zerocopy::little_endian::U32;
use zerocopy::{IntoBytes, TryFromBytes};

#[derive(Debug, Clone, Copy, zerocopy::TryFromBytes, zerocopy::IntoBytes, zerocopy::Immutable)]
#[repr(u8)]
pub enum Format {
    QoiRgba = 1,
}

#[derive(Debug, Clone, Copy, zerocopy::TryFromBytes, zerocopy::IntoBytes, zerocopy::Immutable)]
#[repr(C)]
pub struct Header {
    pub format: Format,
    pub width: U32,
    pub height: U32,
}

pub fn encode(texture: &Texture<'_>) -> Vec<u8> {
    qoi_rgba_encode(texture)
}

pub fn decode(data: &[u8]) -> Result<Texture<'static>, Error> {
    let (header, data) = Header::try_read_from_prefix(data).map_err(|_| Error::UnknownFormat)?;

    match header.format {
        Format::QoiRgba => qoi_rgba_decode(&header, data),
    }
}

fn qoi_rgba_encode(texture: &Texture<'_>) -> Vec<u8> {
    assert_eq!(texture.format, TextureFormat::Rgba8NonlinearSrgb);

    let max_len =
        size_of::<Header>() + (texture.resolution.x as usize) * (texture.resolution.y as usize) * 5;

    let mut output = vec![0; max_len];

    let header = Header {
        format: Format::QoiRgba,
        width: U32::new(texture.resolution.x),
        height: U32::new(texture.resolution.y),
    };

    output[..size_of::<Header>()].copy_from_slice(header.as_bytes());

    let mut index = [rapid_qoi::Pixel::new(); 64];
    let mut px_prev = rapid_qoi::Pixel::new_opaque();
    let mut run = 0usize;

    let mut output_offset = size_of::<Header>();

    for row in texture.data.chunks(texture.row_stride) {
        output_offset += rapid_qoi::Qoi::encode_range::<4>(
            &mut index,
            &mut px_prev,
            &mut run,
            &row[..(texture.resolution.x as usize) * 4],
            &mut output[output_offset..],
        )
        .unwrap();
    }

    output.truncate(output_offset);
    output
}

fn qoi_rgba_decode(header: &Header, data: &[u8]) -> Result<Texture<'static>, Error> {
    let qoi = rapid_qoi::Qoi {
        width: header.width.get(),
        height: header.height.get(),
        colors: rapid_qoi::Colors::SrgbLinA,
    };

    let mut output = vec![0; (qoi.width as usize) * (qoi.height as usize) * 4];

    qoi.decode_skip_header(data, &mut output)
        .map_err(|_| Error::UnknownFormat)?;

    Ok(Texture {
        resolution: UVec2 {
            x: qoi.width,
            y: qoi.height,
        },
        format: TextureFormat::Rgba8NonlinearSrgb,
        data: output.into(),
        row_stride: (qoi.width as usize) * 4,
    })
}
