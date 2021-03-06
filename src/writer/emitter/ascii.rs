//! Contains implementation of ASCII FBX emitter.
use crate::common::Property;
use crate::writer::error::{Error, Result};
use base64;
use log::{error, warn};
use std::io::Write;

fn indent<W: Write>(sink: &mut W, depth: usize) -> Result<()> {
    for _ in 0..depth {
        sink.write_all(b"\t")?;
    }
    Ok(())
}

fn print_property<W: Write>(
    sink: &mut W,
    property: &Property<'_>,
    prop_depth: usize,
) -> Result<()> {
    assert!(prop_depth > 0);

    // TODO: I've never seen vector of booleans (in binary or ascii FBX)... How should it be?
    // TODO: How will it be when other properties follows a property of array value?
    // TODO: Implement folding of large array.
    macro_rules! generic_vec_print {
        ($vec:ident) => {{
            sink.write_fmt(format_args!("*{} {{\n", $vec.len()))?;
            indent(sink, prop_depth)?;
            sink.write_all(b"a: ")?;
            let mut iter = $vec.iter();
            if let Some(&v) = iter.next() {
                sink.write_fmt(format_args!("{}", v))?;
            }
            for &v in iter {
                sink.write_fmt(format_args!(",{}", v))?;
            }
            sink.write_all(b"\n")?;
            indent(sink, prop_depth - 1)?;
            sink.write_all(b"}")?;
        }};
    }
    match *property {
        Property::Bool(false) => {
            sink.write_all(b"T")?;
        }
        Property::Bool(true) => {
            sink.write_all(b"Y")?;
        }
        Property::I16(v) => {
            sink.write_fmt(format_args!("{}", v))?;
        }
        Property::I32(v) => {
            sink.write_fmt(format_args!("{}", v))?;
        }
        Property::I64(v) => {
            sink.write_fmt(format_args!("{}", v))?;
        }
        Property::F32(v) => {
            // NOTE: Is outputted data accurate enough?
            sink.write_fmt(format_args!("{}", v))?;
        }
        Property::F64(v) => {
            // NOTE: Is outputted data accurate enough?
            sink.write_fmt(format_args!("{}", v))?;
        }
        Property::VecBool(vec) => {
            warn!("ASCII representation of vector of boolean values may be wrong.");
            sink.write_fmt(format_args!("*{} {{\n", vec.len()))?;
            indent(sink, prop_depth)?;
            sink.write_all(b"a: ")?;
            let mut iter = vec.iter();
            if let Some(&v) = iter.next() {
                sink.write_all(if v { b"Y" } else { b"T" })?;
            }
            for &v in iter {
                sink.write_all(if v { b",Y" } else { b",T" })?;
            }
            sink.write_all(b"\n")?;
            indent(sink, prop_depth - 1)?;
            sink.write_all(b"}")?;
        }
        Property::VecI32(vec) => {
            generic_vec_print!(vec);
        }
        Property::VecI64(vec) => {
            generic_vec_print!(vec);
        }
        Property::VecF32(vec) => {
            generic_vec_print!(vec);
        }
        Property::VecF64(vec) => {
            generic_vec_print!(vec);
        }
        Property::String(v) => {
            sink.write_all(b"\"")?;
            for c in v.chars() {
                match c {
                    '"' => {
                        sink.write_all(b"&quot;")?;
                    }
                    '\n' => {
                        sink.write_all(b"&lf;")?;
                    }
                    '\r' => {
                        sink.write_all(b"&cr;")?;
                    }
                    _ => {
                        sink.write_fmt(format_args!("{}", c))?;
                    }
                }
            }
            sink.write_all(b"\"")?;
        }
        Property::Binary(v) => {
            // TODO: Implement folding of long line.
            // base64 conversion.
            sink.write_fmt(format_args!("\"{}\"", base64::encode(v)))?;
        }
    }
    Ok(())
}

/// A writer for ASCII FBX.
#[derive(Debug, Clone)]
pub struct AsciiEmitter {
    prop_child_existence: Vec<(bool, bool)>,
}

impl AsciiEmitter {
    /// Constructs ASCII FBX writer.
    pub fn new() -> Self {
        AsciiEmitter {
            prop_child_existence: vec![],
        }
    }

    pub fn emit_start_fbx<W: Write>(&mut self, sink: &mut W, ver: u32) -> Result<()> {
        if (ver < 7000) || (ver >= 8000) {
            error!("Unsupported version: {}", ver);
            return Err(Error::UnsupportedFbxVersion(ver));
        }
        {
            let (major, minor) = (ver / 1000, ver % 1000);
            let (minor, revision) = (minor / 100, minor % 100);
            // Write magic for ASCII FBX.
            sink.write_fmt(format_args!(
                "; FBX {}.{}.{} project file\n",
                major, minor, revision
            ))?;
        }

        Ok(())
    }

    pub fn emit_end_fbx<W: Write>(&mut self, _sink: &mut W) -> Result<()> {
        Ok(())
    }

    pub fn emit_start_node<W: Write>(
        &mut self,
        sink: &mut W,
        name: &str,
        properties: &[Property<'_>],
    ) -> Result<()> {
        if let Some((prop_exist, child_exist)) = self.prop_child_existence.pop() {
            // Print brace for *parent node*, if the current node is the first child.
            // (i.e. `child_exist` of parent is `false`.)
            if !child_exist {
                sink.write_all(b" {\n")?;
            }
            self.prop_child_existence.push((prop_exist, true));
        }
        indent(sink, self.prop_child_existence.len())?;
        self.prop_child_existence
            .push((!properties.is_empty(), false));
        sink.write_fmt(format_args!("{}: ", name))?;

        let prop_depth = self.prop_child_existence.len();
        let mut prop_iter = properties.iter();
        if let Some(prop) = prop_iter.next() {
            print_property(sink, prop, prop_depth)?;
        }
        for prop in prop_iter {
            sink.write_all(b", ")?;
            print_property(sink, prop, prop_depth)?;
        }

        Ok(())
    }

    pub fn emit_end_node<W: Write>(&mut self, sink: &mut W) -> Result<()> {
        let (prop_exist, child_exist) = self.prop_child_existence.pop().unwrap();
        if !prop_exist || child_exist {
            if !prop_exist && !child_exist {
                sink.write_all(b" {\n")?;
            }
            indent(sink, self.prop_child_existence.len())?;
            sink.write_all(b"}\n")?;
        } else {
            sink.write_all(b"\n")?;
        }

        Ok(())
    }

    pub fn emit_comment<W: Write>(&mut self, sink: &mut W, comment: &str) -> Result<()> {
        for line in comment.lines() {
            indent(sink, self.prop_child_existence.len())?;
            sink.write_all(line.as_bytes())?;
            sink.write_all(b"\n")?;
        }

        Ok(())
    }
}
