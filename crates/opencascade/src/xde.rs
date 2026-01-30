//! XDE (Extended Data Exchange) document support for shapes with colors and metadata.
//!
//! This module provides the [`XdeDocument`] type which allows working with shapes
//! that have associated colors, materials, and other metadata. It supports:
//!
//! - Reading STEP files with colors preserved via [`XdeDocument::read_step`]
//! - Setting colors on shapes programmatically
//! - Exporting to glTF/GLB format with colors via [`XdeDocument::write_gltf`] and [`XdeDocument::write_glb`]
//!
//! # Example: STEP to glTF conversion with colors
//!
//! ```no_run
//! use opencascade::xde::XdeDocument;
//!
//! let doc = XdeDocument::read_step("input.step").unwrap();
//! doc.write_glb("output.glb").unwrap();
//! ```
//!
//! # Example: Creating shapes with colors
//!
//! ```no_run
//! use opencascade::primitives::Shape;
//! use opencascade::xde::XdeDocument;
//!
//! let cube = Shape::cube(10.0, 10.0, 10.0);
//! let doc = XdeDocument::new();
//! let label = doc.add_shape(&cube);
//! doc.set_color_rgb(&label, 1.0, 0.0, 0.0); // Red
//! doc.write_glb("red_cube.glb").unwrap();
//! ```

use crate::primitives::Shape;
use crate::Error;
use opencascade_sys::ffi;
use std::path::Path;

/// A label identifying a shape within an XDE document.
///
/// Labels are returned when adding shapes to a document and can be used
/// to set colors or other attributes on specific shapes.
pub struct ShapeLabel {
    inner: cxx::UniquePtr<ffi::TDF_Label>,
}

/// An XDE (Extended Data Exchange) document that holds shapes with colors and metadata.
///
/// XDE documents are the standard way to work with CAD data that includes
/// more than just geometry - colors, materials, names, layers, and assembly
/// structure can all be preserved.
///
/// The primary use case is converting STEP files to glTF while preserving colors:
///
/// ```no_run
/// use opencascade::xde::XdeDocument;
///
/// let doc = XdeDocument::read_step("model.step")?;
/// doc.write_glb("model.glb")?;
/// # Ok::<(), opencascade::Error>(())
/// ```
pub struct XdeDocument {
    inner: cxx::UniquePtr<ffi::HandleTDocStd_Document>,
}

impl XdeDocument {
    /// Create a new empty XDE document.
    pub fn new() -> Self {
        Self {
            inner: ffi::XCAFApp_NewDocument(),
        }
    }

    /// Read a STEP file preserving colors, names, and layers.
    ///
    /// This uses `STEPCAFControl_Reader` internally, which preserves all
    /// XDE attributes from the STEP file including colors and assembly structure.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::xde::XdeDocument;
    ///
    /// let doc = XdeDocument::read_step("colored_model.step")?;
    /// // Colors are automatically preserved when exporting
    /// doc.write_glb("colored_model.glb")?;
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn read_step<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = ffi::read_step_with_colors(path.as_ref().to_string_lossy().to_string());
        if inner.is_null() {
            return Err(Error::StepCafReadFailed);
        }
        Ok(Self { inner })
    }

    /// Add a shape to the document, returning its label for attribute assignment.
    ///
    /// The returned [`ShapeLabel`] can be used to set colors or other attributes
    /// on the shape.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::primitives::Shape;
    /// use opencascade::xde::XdeDocument;
    ///
    /// let sphere = Shape::sphere(5.0);
    /// let doc = XdeDocument::new();
    /// let label = doc.add_shape(&sphere);
    /// doc.set_color_rgb(&label, 0.0, 1.0, 0.0); // Green
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn add_shape(&self, shape: &Shape) -> ShapeLabel {
        let shape_tool = ffi::XCAFDoc_DocumentTool_ShapeTool(&self.inner);
        let label = ffi::XCAFDoc_ShapeTool_AddShape(&shape_tool, shape.inner());
        ShapeLabel { inner: label }
    }

    /// Set an RGB color on a shape.
    ///
    /// Color components should be in the range 0.0 to 1.0.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::primitives::Shape;
    /// use opencascade::xde::XdeDocument;
    ///
    /// let cube = Shape::cube(10.0, 10.0, 10.0);
    /// let doc = XdeDocument::new();
    /// let label = doc.add_shape(&cube);
    /// doc.set_color_rgb(&label, 1.0, 0.5, 0.0); // Orange
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn set_color_rgb(&self, label: &ShapeLabel, r: f64, g: f64, b: f64) {
        let color_tool = ffi::XCAFDoc_DocumentTool_ColorTool(&self.inner);
        ffi::XCAFDoc_ColorTool_SetColor_RGB(&color_tool, &label.inner, r, g, b);
    }

    /// Set an RGBA color on a shape (with transparency).
    ///
    /// Color components should be in the range 0.0 to 1.0.
    /// Alpha of 1.0 is fully opaque, 0.0 is fully transparent.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::primitives::Shape;
    /// use opencascade::xde::XdeDocument;
    ///
    /// let cube = Shape::cube(10.0, 10.0, 10.0);
    /// let doc = XdeDocument::new();
    /// let label = doc.add_shape(&cube);
    /// doc.set_color_rgba(&label, 0.0, 0.0, 1.0, 0.5); // Semi-transparent blue
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn set_color_rgba(&self, label: &ShapeLabel, r: f64, g: f64, b: f64, a: f64) {
        let color_tool = ffi::XCAFDoc_DocumentTool_ColorTool(&self.inner);
        ffi::XCAFDoc_ColorTool_SetColor_RGBA(&color_tool, &label.inner, r, g, b, a);
    }

    /// Write the document to a glTF file (.gltf with separate .bin).
    ///
    /// This exports all shapes in the document along with their colors
    /// and materials. The output consists of a .gltf JSON file and a
    /// separate .bin file containing the mesh data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::xde::XdeDocument;
    ///
    /// let doc = XdeDocument::read_step("model.step")?;
    /// doc.write_gltf("model.gltf")?;
    /// // Creates: model.gltf and model.bin
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn write_gltf<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.write_gltf_impl(path, false)
    }

    /// Write the document to a binary glTF file (.glb).
    ///
    /// This exports all shapes in the document along with their colors
    /// and materials into a single binary file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use opencascade::xde::XdeDocument;
    ///
    /// let doc = XdeDocument::read_step("model.step")?;
    /// doc.write_glb("model.glb")?;
    /// # Ok::<(), opencascade::Error>(())
    /// ```
    pub fn write_glb<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.write_gltf_impl(path, true)
    }

    fn write_gltf_impl<P: AsRef<Path>>(&self, path: P, is_binary: bool) -> Result<(), Error> {
        let success = ffi::write_gltf(
            &self.inner,
            path.as_ref().to_string_lossy().to_string(),
            is_binary,
        );
        if success {
            Ok(())
        } else {
            Err(Error::GltfWriteFailed)
        }
    }
}

impl Default for XdeDocument {
    fn default() -> Self {
        Self::new()
    }
}
