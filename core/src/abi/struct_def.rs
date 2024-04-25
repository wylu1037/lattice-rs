use ethabi::ParamType;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclaration {
    pub name: String,
    pub ty: FieldType,
}

/// A field declaration inside a struct
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Represents elementary types, see [`ParamType`]
    ///
    /// Note: tuples will be treated as rust tuples
    Elementary(ParamType),
    /// A non-elementary type field, treated as user-defined struct
    Struct(StructFieldType),
    /// Mapping
    Mapping(Box<MappingType>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MappingType {
    /// key types can be elementary and `bytes` and `string`
    ///
    /// Valid `ParamType` variants are:
    ///     `Address`, `Bytes`, `Int`, `UInt`, `Bool`, `String`, `FixedBytes`,
    key_type: ParamType,
    /// The value type of this mapping
    value_type: FieldType,
}

/// How the type if a struct field is referenced
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructType {
    /// The name of the struct (or rather the name of the rust type)
    name: String,
    /// All previous projections up until the name
    ///
    /// For `MostOuter.Outer.<name>` this is `vec!["MostOuter", "Outer"]`
    projections: Vec<String>,
}

/// Represents the type of field in a struct
#[derive(Debug, Clone, PartialEq)]
pub enum StructFieldType {
    /// A non-elementary type field, represents a user defined struct
    Type(StructType),
    /// Array of user defined type
    Array(Box<StructFieldType>),
    /// Array with fixed size of use defined type
    FixedArray(Box<StructFieldType>, usize),
}