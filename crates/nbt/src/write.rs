use std::{
    io::{self, Write},
    ops::DerefMut,
};

use crate::{string::NbtStr, ArrayTag, Tag};

/// An object used to write data which is in a form of NBT.
///
/// This has *plain* API which means that no state is preserved
/// and it is the caller's responsibility to maintain NBT invariants.
pub trait NbtWrite {
    type Fork<'f>: NbtWrite
    where
        Self: 'f;

    fn fork(&mut self) -> Self::Fork<'_>;

    fn is_human_readable(&self) -> bool;

    fn write_tag_end(&mut self) -> io::Result<()> {
        self.write_tag(Tag::End)
    }

    fn write_tag(&mut self, tag: Tag) -> io::Result<()>;

    fn write_byte(&mut self, value: i8) -> io::Result<()>;

    fn write_short(&mut self, value: i16) -> io::Result<()>;

    fn write_int(&mut self, value: i32) -> io::Result<()>;

    fn write_long(&mut self, value: i64) -> io::Result<()>;

    fn write_float(&mut self, value: f32) -> io::Result<()>;

    fn write_double(&mut self, value: f64) -> io::Result<()>;

    fn write_string(&mut self, value: &NbtStr) -> io::Result<()>;

    /// # Panics
    ///
    /// If `size` is negative.
    fn start_array(&mut self, tag: ArrayTag, len: i32) -> io::Result<()>;

    /// # Panics
    ///
    /// If the implementation detects that this is not balanced with [`Self::start_array()`].
    fn end_array(&mut self);

    /// # Panics
    ///
    /// If `size` is negative.
    fn start_list(&mut self, tag: Tag, len: i32) -> io::Result<()>;

    /// # Panics
    ///
    /// If the implementation detects that this is not balanced with [`Self::start_list()`].
    fn end_list(&mut self) -> io::Result<()>;

    fn start_compound(&mut self, name: &NbtStr) -> io::Result<()>;

    /// # Panics
    ///
    /// If the implementation detects that this is not balanced with [`Self::start_compound()`].
    fn end_compound(&mut self) -> io::Result<()>;
}

impl<D: DerefMut<Target = T>, T: NbtWrite> NbtWrite for D {
    type Fork<'f> = &'f mut Self
    where
        Self: 'f;

    fn fork(&mut self) -> Self::Fork<'_> {
        self
    }

    fn is_human_readable(&self) -> bool {
        T::is_human_readable(self)
    }

    fn write_tag_end(&mut self) -> io::Result<()> {
        T::write_tag_end(self)
    }

    fn write_tag(&mut self, tag: Tag) -> io::Result<()> {
        T::write_tag(self, tag)
    }

    fn write_byte(&mut self, value: i8) -> io::Result<()> {
        T::write_byte(self, value)
    }

    fn write_short(&mut self, value: i16) -> io::Result<()> {
        T::write_short(self, value)
    }

    fn write_int(&mut self, value: i32) -> io::Result<()> {
        T::write_int(self, value)
    }

    fn write_long(&mut self, value: i64) -> io::Result<()> {
        T::write_long(self, value)
    }

    fn write_float(&mut self, value: f32) -> io::Result<()> {
        T::write_float(self, value)
    }

    fn write_double(&mut self, value: f64) -> io::Result<()> {
        T::write_double(self, value)
    }

    fn write_string(&mut self, value: &NbtStr) -> io::Result<()> {
        T::write_string(self, value)
    }

    fn start_array(&mut self, tag: ArrayTag, len: i32) -> io::Result<()> {
        T::start_array(self, tag, len)
    }

    fn end_array(&mut self) {
        T::end_array(self)
    }

    fn start_list(&mut self, tag: Tag, len: i32) -> io::Result<()> {
        T::start_list(self, tag, len)
    }

    fn end_list(&mut self) -> io::Result<()> {
        T::end_list(self)
    }

    fn start_compound(&mut self, name: &NbtStr) -> io::Result<()> {
        T::start_compound(self, name)
    }

    fn end_compound(&mut self) -> io::Result<()> {
        T::end_compound(self)
    }
}

pub struct BinaryNbtWriter<W: ?Sized>(W);

impl<W> BinaryNbtWriter<W> {
    pub fn new(writer: W) -> Self {
        Self(writer)
    }
}

impl<W: Write> NbtWrite for BinaryNbtWriter<W> {
    type Fork<'f> = &'f mut Self
    where
        Self: 'f;
    fn fork(&mut self) -> Self::Fork<'_> {
        self
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    fn write_tag(&mut self, tag: Tag) -> io::Result<()> {
        self.0.write_all(&tag.to_be_bytes())?;
        Ok(())
    }

    fn write_byte(&mut self, value: i8) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_short(&mut self, value: i16) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_int(&mut self, value: i32) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_long(&mut self, value: i64) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_float(&mut self, value: f32) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_double(&mut self, value: f64) -> io::Result<()> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    fn write_string(&mut self, value: &NbtStr) -> io::Result<()> {
        value.write(&mut self.0)?;
        Ok(())
    }

    fn start_array(&mut self, tag: ArrayTag, len: i32) -> io::Result<()> {
        assert!(len >= 0, "len should be non-negative");

        self.0.write_all(&tag.to_be_bytes())?;
        self.0.write_all(&len.to_be_bytes())?;
        Ok(())
    }

    fn end_array(&mut self) {}

    fn start_list(&mut self, tag: Tag, len: i32) -> io::Result<()> {
        assert!(len >= 0, "len should be non-negative");

        self.0.write_all(&tag.to_be_bytes())?;
        self.0.write_all(&len.to_be_bytes())?;
        Ok(())
    }

    fn end_list(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn start_compound(&mut self, name: &NbtStr) -> io::Result<()> {
        self.write_string(name)
    }

    fn end_compound(&mut self) -> io::Result<()> {
        self.write_tag_end()
    }
}
