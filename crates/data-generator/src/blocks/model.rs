use indexmap::IndexMap;

use crate::name::Name;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Id(pub u32);

#[derive(Debug, Clone)]
pub struct Block {
    pub properties: IndexMap<Name, PropertyVariants>,
    pub states: States,
}

#[derive(Debug, Clone)]
pub struct States {
    pub states: IndexMap<Id, State>,
    pub default: Id,
}

impl States {
    pub fn default(&self) -> (&Id, &State) {
        (&self.default, self.states.get(&self.default).unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub properties: IndexMap<Name, PropertyVariant>,
}

#[derive(Debug, Clone)]
pub enum PropertyVariant {
    Bool(bool),
    Numeric(u8),
    Enum(Name),
}

impl PropertyVariant {
    pub fn from(variant_raw: impl Into<String>, variants: &PropertyVariants) -> PropertyVariant {
        let variant_raw = variant_raw.into();
        match variants {
            PropertyVariants::Bool => {
                let b = variant_raw.parse().unwrap();
                PropertyVariant::Bool(b)
            }
            PropertyVariants::Numeric(_) => {
                let n = variant_raw.parse().unwrap();
                PropertyVariant::Numeric(n)
            }
            PropertyVariants::Enum(_) => {
                let var = Name::from_snake_case(variant_raw);
                PropertyVariant::Enum(var)
            }
        }
    }

    pub fn variant(&self) -> Name {
        match self {
            Self::Enum(name) => name.clone(),
            Self::Bool(false) => Name::new("false", "False"),
            Self::Bool(true) => Name::new("true", "True"),
            Self::Numeric(n) => {
                let roman = convert_into_roman(*n);
                Name::new(roman.to_lowercase(), roman.as_str())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PropertyVariants {
    Bool,
    Numeric(Vec<u8>),
    Enum(Vec<Name>),
}

impl PropertyVariants {
    pub fn as_enum(&self) -> Vec<(Name, Option<u8>)> {
        type One = PropertyVariant;
        type Many = PropertyVariants;

        macro_rules! one_variant {
            ($src:expr, $one:expr, $option:expr) => {
                $src.iter()
                    .cloned()
                    .map(|n| ($one(n.clone()).variant(), $option(n)))
                    .collect::<Vec<(Name, Option<u8>)>>()
            };
        }

        match self {
            Many::Bool => one_variant!(vec![true, false], One::Bool, |_| None),
            Many::Numeric(vars) => one_variant!(vars, One::Numeric, Some),
            Many::Enum(vars) => one_variant!(vars, One::Enum, |_| None),
        }
    }
}

fn convert_into_roman(n: u8) -> String {
    use numerals::roman::Roman;
    match n {
        0 => "O".to_string(),
        _ => format!("{:X}", Roman::from(n as i16)),
    }
}
