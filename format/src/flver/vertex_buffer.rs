use std::{marker::PhantomData, mem::size_of};

use bytemuck::Pod;
use byteorder::ByteOrder;
use zerocopy::{FromBytes, FromZeroes, U32};

use crate::{flver::header::FlverHeaderPart, io_ext::zerocopy::Padding};

#[derive(Debug, FromBytes, FromZeroes)]
#[allow(unused)]
#[repr(packed)]
pub struct VertexBuffer<O: ByteOrder> {
    pub buffer_index: U32<O>,
    pub layout_index: U32<O>,
    pub vertex_size: U32<O>,
    pub vertex_count: U32<O>,
    padding0: Padding<8>,
    pub buffer_length: U32<O>,
    pub buffer_offset: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for VertexBuffer<O> {}

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(packed)]
#[allow(unused)]
pub struct VertexBufferLayout<O: ByteOrder> {
    pub(crate) member_count: U32<O>,
    padding0: Padding<8>,
    pub(crate) member_offset: U32<O>,
}

impl<O: ByteOrder> FlverHeaderPart for VertexBufferLayout<O> {}

#[derive(Debug, FromBytes, FromZeroes)]
#[repr(packed)]
#[allow(unused)]
pub struct VertexBufferAttribute<O: ByteOrder> {
    pub unk0: U32<O>,
    pub struct_offset: U32<O>,
    pub format_id: U32<O>,
    pub semantic_id: U32<O>,
    pub index: U32<O>,
}

impl<O: ByteOrder> VertexBufferAttribute<O> {
    pub fn format(&self) -> VertexAttributeFormat {
        VertexAttributeFormat::from(self.format_id.get())
    }

    pub fn semantic(&self) -> VertexAttributeSemantic {
        VertexAttributeSemantic::from(self.semantic_id.get())
    }
}

impl<O: ByteOrder> FlverHeaderPart for VertexBufferAttribute<O> {}

pub enum VertexAttributeAccessor<'a> {
    Float2(VertexAttributeIter<'a, [f32; 2]>),
    Float3(VertexAttributeIter<'a, [f32; 3]>),
    Float4(VertexAttributeIter<'a, [f32; 4]>),
    Byte4A(VertexAttributeIter<'a, [u8; 4]>),
    Byte4B(VertexAttributeIter<'a, [u8; 4]>),
    Short2ToFloat2(VertexAttributeIter<'a, [u16; 2]>),
    Byte4C(VertexAttributeIter<'a, [u8; 4]>),
    UV(VertexAttributeIter<'a, [f32; 2]>),
    // TODO: get the last 2 components of this
    UVPair(VertexAttributeIter<'a, [f32; 2]>),
    Short4ToFloat4A(VertexAttributeIter<'a, [u16; 4]>),
    Short4ToFloat4B(VertexAttributeIter<'a, [u16; 4]>),
}

pub struct VertexAttributeIter<'a, T: Pod> {
    buffer: &'a [u8],
    attribute_data_offset: usize,
    attribute_data_end: usize,
    vertex_size: usize,
    _phantom: PhantomData<T>,
}

// TODO: this doesn't support endian sensitive reading like the rest of the FLVER parser.
impl<'a, T: Pod> VertexAttributeIter<'a, T> {
    pub fn new(
        buffer: &'a [u8],
        vertex_size: usize,
        vertex_offset: usize,
    ) -> VertexAttributeIter<'a, T> {
        let attribute_data_offset = vertex_offset;
        let attribute_data_end = attribute_data_offset + size_of::<T>();

        Self {
            buffer,
            attribute_data_offset,
            attribute_data_end,
            vertex_size,
            _phantom: Default::default(),
        }
    }
}

impl<'a, T: Pod> ExactSizeIterator for VertexAttributeIter<'a, T> {}

impl<'a, T: Pod> Iterator for VertexAttributeIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }

        let attribute_byte_data = &self.buffer[self.attribute_data_offset..self.attribute_data_end];
        let data: &T = bytemuck::from_bytes(attribute_byte_data);

        self.buffer = &self.buffer[self.vertex_size..];

        Some(*data)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() / self.vertex_size;
        (remaining, Some(remaining))
    }
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq)]
// TODO: these come from soulsformats and probably have documented
// names in dx12
pub enum VertexAttributeFormat {
    Float2 = 0x1,
    Float3 = 0x2,
    Float4 = 0x3,
    Byte4A = 0x10,
    Byte4B = 0x11,
    Short2ToFloat2 = 0x12,

    // int to float 127
    Byte4C = 0x13,
    UV = 0x15,

    // int to float
    UVPair = 0x16,
    ShortBoneIndices = 0x18,
    Short4ToFloat4A = 0x1A,
    Short4ToFloat4B = 0x2E,
    Byte4E = 0x2F,
    EdgeCompressed = 0xF0,
}

impl VertexAttributeFormat {
    pub fn datum_size(&self) -> usize {
        match self {
            VertexAttributeFormat::Float2
            | VertexAttributeFormat::Float3
            | VertexAttributeFormat::Float4
            | VertexAttributeFormat::UV
            | VertexAttributeFormat::UVPair => 4,
            VertexAttributeFormat::Byte4A
            | VertexAttributeFormat::Byte4B
            | VertexAttributeFormat::Byte4C
            | VertexAttributeFormat::Byte4E => 1,
            VertexAttributeFormat::Short2ToFloat2
            | VertexAttributeFormat::ShortBoneIndices
            | VertexAttributeFormat::Short4ToFloat4A
            | VertexAttributeFormat::Short4ToFloat4B => 2,
            _ => unimplemented!(),
        }
    }
    pub fn dimensions(&self) -> usize {
        match self {
            VertexAttributeFormat::Float2 => 2,
            VertexAttributeFormat::Float3 => 3,
            VertexAttributeFormat::Float4 => 4,
            VertexAttributeFormat::Byte4A => 4,
            VertexAttributeFormat::Byte4B => 4,
            VertexAttributeFormat::Short2ToFloat2 => 2,
            VertexAttributeFormat::Byte4C => 4,
            VertexAttributeFormat::UV => 2,
            VertexAttributeFormat::UVPair => 4,
            VertexAttributeFormat::ShortBoneIndices => 4,
            VertexAttributeFormat::Short4ToFloat4A => 4,
            VertexAttributeFormat::Short4ToFloat4B => 4,
            VertexAttributeFormat::Byte4E => 4,
            VertexAttributeFormat::EdgeCompressed => unimplemented!(),
        }
    }
}

impl From<u32> for VertexAttributeFormat {
    fn from(value: u32) -> Self {
        match value {
            0x1 => Self::Float2,
            0x2 => Self::Float3,
            0x3 => Self::Float4,
            0x10 => Self::Byte4A,
            0x11 => Self::Byte4B,
            0x12 => Self::Short2ToFloat2,
            0x13 => Self::Byte4C,
            0x15 => Self::UV,
            0x16 => Self::UVPair,
            0x18 => Self::ShortBoneIndices,
            0x1A => Self::Short4ToFloat4A,
            0x2E => Self::Short4ToFloat4B,
            0x2F => Self::Byte4E,
            0xF0 => Self::EdgeCompressed,
            _ => panic!("Unknown storage type {}", value),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VertexAttributeSemantic {
    Position,
    BoneWeights,
    BoneIndices,
    Normal,
    UV,
    Tangent,
    Bitangent,
    VertexColor,
}

impl From<u32> for VertexAttributeSemantic {
    fn from(value: u32) -> Self {
        match value {
            0x0 => Self::Position,
            0x1 => Self::BoneWeights,
            0x2 => Self::BoneIndices,
            0x3 => Self::Normal,
            0x5 => Self::UV,
            0x6 => Self::Tangent,
            0x7 => Self::Bitangent,
            0xA => Self::VertexColor,
            _ => panic!("Unknown member type {}", value),
        }
    }
}
