use std::io::Read;

const PNG_FORMAT: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
const IHDR_TYPE: [u8; 4] = [73, 72, 68, 82];

fn main() {
    let mut file =
        std::fs::File::open("./test/resources/2x2.png").expect("Failed loading the file.");
    let mut buffer: Vec<u8> = Vec::new();
    let _ = file.read_to_end(&mut buffer);

    let is_png = &buffer[..8] == PNG_FORMAT;

    if is_png {
        // println!("{:?}", &buffer[8..]);
        // http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html

        // Length 4 bytes
        // Chunk type 4 bytes
        // Chunk data length = Length
        //    Width:              4 bytes
        //    Height:             4 bytes
        //    Bit depth:          1 byte
        //    Color type:         1 byte
        //    Compression method: 1 byte
        //    Filter method:      1 byte
        //    Interlace method:   1 byte
        // CRC 4 bytes

        let mut offset = 8;
        let mut chunks = vec![];

        while offset < buffer.len() {
            let chunk = read_chunk(&buffer, &mut offset);

            chunks.push(chunk);
        }

        println!("{:?}", chunks);
    }
}

fn read_chunk(buffer: &Vec<u8>, offset: &mut usize) -> PngChunk {
    let length = u32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;

    let chunk_type = String::from_utf8(buffer[*offset..*offset + 4].to_vec()).unwrap();
    *offset += 4;

    let data = buffer[*offset..*offset + length as usize].to_vec();
    *offset += length as usize;

    let crc = u32::from_be_bytes(buffer[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;

    return PngChunk {
        length,
        chunk_type,
        data,
        crc,
    };
}

fn get_png_metadata(buffer: &Vec<u8>) -> Result<PngMetadata, String> {
    let mut offset = 8;
    let length = u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap());
    offset += 4;

    let chunk_type = &buffer[offset..offset + 4];
    offset += 4;

    if chunk_type != IHDR_TYPE {
        return Err(format!("Invalid chunk type: {:?}", chunk_type));
    }

    let chunk_data = &buffer[offset..offset + length as usize];
    offset += length as usize;

    let width = u32::from_be_bytes(chunk_data[..4].try_into().unwrap());
    let height = u32::from_be_bytes(chunk_data[4..8].try_into().unwrap());
    let bit_depth = chunk_data[8];
    let color_type = chunk_data[9];
    let compression_method = chunk_data[10];
    let filter_method = chunk_data[11];
    let interlace_method = chunk_data[12];

    offset += 4;

    let png_metadata = PngMetadata {
        offset,
        width,
        height,
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        interlace_method,
    };

    return Ok(png_metadata);
}

#[derive(Debug)]
struct PngMetadata {
    offset: usize,
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug)]
struct PngChunk {
    length: u32,
    chunk_type: String,
    data: Vec<u8>,
    crc: u32,
}
