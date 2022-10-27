use crate::error::*;
use crate::context::*;
use crate::parser::*;
use crate::evaluator::*;
use crate::position::Position;

pub type NativFunction = fn(Vec<V>, &mut Context, &Position, &Vec<&Position>) -> Result<(V, R), E>;

#[derive(Clone)]
pub enum Type {
    Undefined, Any, Int, Float, Bool, String, NativFunction, Function,
    Addr, Closure,
    Union(Vec<Type>), Type
}
impl Type {
    pub fn cast(&self, value: &V) -> V {
        match self {
            Self::Undefined => V::Null,
            Self::Any => value.clone(),
            Self::Int => match value {
                V::Null => V::Int(0),
                V::Int(v) => V::Int(*v),
                V::Float(v) => V::Int(*v as i64),
                V::Bool(v) => V::Int(*v as i64),
                _ => V::Null
            }
            Self::Float => match value {
                V::Null => V::Float(0.0),
                V::Int(v) => V::Float(*v as f64),
                V::Float(v) => V::Float(*v),
                V::Bool(v) => V::Float((*v as i64) as f64),
                _ => V::Null
            }
            Self::Bool => match value {
                V::Null => V::Bool(false),
                V::Int(v) => V::Bool(*v != 0),
                V::Float(v) => V::Bool(*v != 0.0),
                V::Bool(v) => V::Bool(*v),
                _ => V::Null
            }
            Self::String => V::String(value.to_string()),
            Self::Addr => V::Addr(value.to_string()),
            Self::Type => V::Type(value.typ()),
            _ => V::Null
        }
    }
}
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Any => write!(f, "any"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::String => write!(f, "str"),
            Self::NativFunction => write!(f, "nativ-function"),
            Self::Function => write!(f, "function"),
            Self::Addr => write!(f, "addr"),
            Self::Closure => write!(f, "closure"),
            Self::Union(types) => write!(f, "{}", types.iter().map(|x| x.to_string()).collect::<Vec<String>>().join("|")),
            Self::Type => write!(f, "type"),
        }
    }
}
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        if let Self::Any = self { return true }
        if let Self::Any = other { return true }
        if let Self::Union(types) = self {
            if let Self::Union(other_types) = other {
                for typ in types {
                    if !other_types.contains(typ) { return false }
                }
                return true
            }
            return types.contains(other)
        }
        if let Self::Union(types) = other {
            if let Self::Union(other_types) = self {
                for typ in types {
                    if !other_types.contains(typ) { return false }
                }
                return true
            }
            return types.contains(self)
        }
        match (self, other) {
            (Self::Undefined, Self::Undefined) => true,
            (Self::Int, Self::Int) => true,
            (Self::Float, Self::Float) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::String, Self::String) => true,
            (Self::Addr, Self::Addr) => true,
            (Self::Closure, Self::Closure) => true,
            (Self::NativFunction, Self::NativFunction) => true,
            (Self::Function, Self::Function) => true,
            (Self::Type, Self::Type) => true,
            _ => false
        }
    }
}

#[derive(Clone)]
pub enum V {
    Null, Wirldcard, Int(i64), Float(f64), Bool(bool), String(String),
    Addr(String), Closure(Node),
    NativFunction(Vec<Type>, NativFunction), Function(Vec<Node>, Node),
    Type(Type)
}
impl V {
    pub fn typ(&self) -> Type {
        match self {
            Self::Null => Type::Undefined,
            Self::Wirldcard => Type::Any,
            Self::Int(_) => Type::Int,
            Self::Float(_) => Type::Float,
            Self::Bool(_) => Type::Bool,
            Self::String(_) => Type::String,
            Self::Addr(_) => Type::Addr,
            Self::Closure(_) => Type::Closure,
            Self::NativFunction(_, _) => Type::NativFunction,
            Self::Function(_, _) => Type::Function,
            Self::Type(_) => Type::Type,
        }
    }
    pub fn add(&self, other: &V) -> Option<V> {
        match self {
            Self::Int(v1) => match other {
                Self::Int(v2) => Some(V::Int(v1 + v2)),
                Self::Float(v2) => Some(V::Float((*v1 as f64) + v2)),
                _ => None
            }
            Self::Float(v1) => match other {
                Self::Float(v2) => Some(V::Float(v1 + v2)),
                Self::Int(v2) => Some(V::Float(v1 + (*v2 as f64))),
                _ => None
            }
            Self::String(v1) => match other {
                Self::String(v2) => Some(V::String(v1.to_owned() + v2)),
                _ => None
            }
            _ => None
        }
    }
    pub fn sub(&self, other: &V) -> Option<V> {
        match self {
            Self::Int(v1) => match other {
                Self::Int(v2) => Some(V::Int(v1 - v2)),
                Self::Float(v2) => Some(V::Float((*v1 as f64) - v2)),
                _ => None
            }
            Self::Float(v1) => match other {
                Self::Float(v2) => Some(V::Float(v1 - v2)),
                Self::Int(v2) => Some(V::Float(v1 - (*v2 as f64))),
                _ => None
            }
            _ => None
        }
    }
    pub fn mul(&self, other: &V) -> Option<V> {
        match self {
            Self::Int(v1) => match other {
                Self::Int(v2) => Some(V::Int(v1 * v2)),
                Self::Float(v2) => Some(V::Float((*v1 as f64) * v2)),
                _ => None
            }
            Self::Float(v1) => match other {
                Self::Float(v2) => Some(V::Float(v1 * v2)),
                Self::Int(v2) => Some(V::Float(v1 * (*v2 as f64))),
                _ => None
            }
            _ => None
        }
    }
    pub fn div(&self, other: &V) -> Option<V> {
        match self {
            Self::Int(v1) => match other {
                Self::Int(v2) => Some(V::Float((*v1 as f64) / (*v2 as f64))),
                Self::Float(v2) => Some(V::Float((*v1 as f64) / v2)),
                _ => None
            }
            Self::Float(v1) => match other {
                Self::Float(v2) => Some(V::Float(v1 / v2)),
                Self::Int(v2) => Some(V::Float(v1 / (*v2 as f64))),
                _ => None
            }
            _ => None
        }
    }
}
impl std::fmt::Display for V {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::fmt::Debug for V {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Wirldcard => write!(f, "_"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{v}"),
            Self::Addr(v) => write!(f, "@{v}"),
            Self::Closure(v) => write!(f, "#{v}"),
            Self::NativFunction(_, v) => write!(f, "nativ-function:{:?}", v as *const NativFunction),
            Self::Function(_, body) => write!(f, "function:{:?}", body as *const Node),
            Self::Type(typ) => write!(f, "{typ}"),
        }
    }
}
impl PartialEq for V {
    fn eq(&self, other: &Self) -> bool {
        if let Self::Wirldcard = self { return true }
        if let Self::Wirldcard = other { return true }
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Int(v1), Self::Int(v2)) => *v1 == *v2,
            (Self::Int(v1), Self::Float(v2)) => *v1 as f64 == *v2,
            (Self::Float(v1), Self::Int(v2)) => *v1 == *v2 as f64,
            (Self::Float(v1), Self::Float(v2)) => *v1 == *v2,
            (Self::Bool(v1), Self::Bool(v2)) => v1 == v2,
            (Self::String(v1), Self::String(v2)) => v1 == v2,
            _ => false
        }
    }
}