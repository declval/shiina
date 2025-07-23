use crate::bencode::error::Error;
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use std::ops::{AddAssign, MulAssign, Neg};

pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }
}

pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(input);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn peek_byte(&mut self) -> Result<&u8, Error> {
        self.input.iter().next().ok_or(Error::Eof)
    }

    fn next_byte(&mut self) -> Result<&u8, Error> {
        let byte = self.input.iter().next().ok_or(Error::Eof)?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        Err(Error::Syntax)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T, Error>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_byte()? {
            byte @ b'0'..=b'9' => T::from(byte - b'0'),
            _ => return Err(Error::ExpectedInteger),
        };

        loop {
            match self.input.iter().next() {
                Some(byte @ b'0'..=b'9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from(byte - b'0');
                }
                _ => {
                    return Ok(int);
                }
            }
        }
    }

    fn parse_signed<T>(&mut self) -> Result<T, Error>
    where
        T: Neg<Output = T> + AddAssign<T> + MulAssign<T> + From<i8>,
    {
        let neg = if self.peek_byte()? == &b'-' {
            let _ = self.next_byte();
            true
        } else {
            false
        };

        let zero = self.peek_byte()? == &b'0';

        if neg && zero {
            return Err(Error::Syntax);
        }

        let mut int = match self.next_byte()? {
            byte @ b'0'..=b'9' => T::from((byte - b'0') as i8),
            _ => return Err(Error::ExpectedInteger),
        };

        if zero && self.peek_byte()? != &b'e' {
            return Err(Error::Syntax);
        }

        loop {
            match self.input.iter().next() {
                Some(byte @ b'0'..=b'9') => {
                    self.input = &self.input[1..];
                    int *= T::from(10);
                    int += T::from((byte - b'0') as i8);
                }
                _ => {
                    return if neg { Ok(-int) } else { Ok(int) };
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<&'de str, Error> {
        let len = self.parse_unsigned()?;

        if self.next_byte()? != &b':' {
            return Err(Error::ExpectedString);
        }

        if self.input.len() < len {
            return Err(Error::Eof);
        }

        let str = match std::str::from_utf8(&self.input[..len]) {
            Ok(str) => str,
            Err(_) => {
                return Err(Error::ExpectedString);
            }
        };

        self.input = &self.input[len..];

        Ok(str)
    }

    fn parse_bytes(&mut self) -> Result<&'de [u8], Error> {
        let len = self.parse_unsigned()?;

        if self.next_byte()? != &b':' {
            return Err(Error::ExpectedString);
        }

        if self.input.len() < len {
            return Err(Error::Eof);
        }

        let bytes = &self.input[..len];

        self.input = &self.input[len..];

        Ok(bytes)
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_byte()? {
            b'0'..=b'9' => self.deserialize_bytes(visitor),
            b'i' => self.deserialize_i64(visitor),
            b'd' => self.deserialize_map(visitor),
            b'l' => self.deserialize_seq(visitor),
            _ => Err(Error::Syntax),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_signed()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_i8(int)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_signed()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_i16(int)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_signed()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_i32(int)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_signed()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_i64(int)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_unsigned()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_u8(int)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_unsigned()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_u16(int)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_unsigned()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_u32(int)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.peek_byte()? != &b'i' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        let int = self.parse_unsigned()?;

        if self.peek_byte()? != &b'e' {
            return Err(Error::ExpectedInteger);
        }
        self.next_byte()?;

        visitor.visit_u64(int)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s = self.parse_string()?;
        if s.len() > 1 {
            return Err(Error::Message(format!(
                "Expected one character, got {}",
                s.len()
            )));
        }
        visitor.visit_char(s.chars().next().unwrap())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Syntax)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.next_byte()? == &b'l' {
            let value = visitor.visit_seq(EmptyStringSeparated::new(self))?;

            if self.next_byte()? == &b'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.next_byte()? == &b'd' {
            let value = visitor.visit_map(EmptyStringSeparated::new(self))?;

            if self.next_byte()? == &b'e' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.peek_byte()? {
            b'0'..=b'9' => visitor.visit_enum(self.parse_string()?.into_deserializer()),
            b'd' => {
                self.next_byte()?;

                let value = visitor.visit_enum(Enum::new(self))?;

                if self.next_byte()? == &b'e' {
                    Ok(value)
                } else {
                    Err(Error::ExpectedMapEnd)
                }
            }
            _ => Err(Error::ExpectedEnum),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct EmptyStringSeparated<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EmptyStringSeparated<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        EmptyStringSeparated { de }
    }
}

impl<'de> SeqAccess<'de> for EmptyStringSeparated<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.peek_byte()? == &b'e' {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de> MapAccess<'de> for EmptyStringSeparated<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.peek_byte()? == &b'e' {
            return Ok(None);
        }

        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de> EnumAccess<'de> for Enum<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.de)?;

        Ok((value, self))
    }
}

impl<'de> VariantAccess<'de> for Enum<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::ExpectedString)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::from_bytes;

    #[test]
    fn test_zero() {
        let input = b"i0e";

        assert_eq!(from_bytes::<i64>(input).unwrap(), 0);
    }

    #[test]
    fn test_negative_zero() {
        let input = b"i-0e";

        assert!(matches!(from_bytes::<i64>(input), Err(Error::Syntax)));
    }

    #[test]
    fn test_leading_zero() {
        let input = b"i01e";

        assert!(matches!(from_bytes::<i64>(input), Err(Error::Syntax)));
    }

    #[test]
    fn test_str() {
        let input = b"3:abc";

        assert_eq!(from_bytes::<String>(input).unwrap(), String::from("abc"));
    }

    #[test]
    fn test_bytes() {
        let input = b"3:abc";

        assert_eq!(
            from_bytes::<&serde_bytes::Bytes>(input).unwrap(),
            serde_bytes::Bytes::new(b"abc")
        );
    }

    #[test]
    fn test_seq() {
        let input = b"l1:a1:b1:ce";

        assert_eq!(
            from_bytes::<Vec<String>>(input).unwrap(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn test_tuple() {
        let input = b"l1:a1:bi1ee";

        assert_eq!(
            from_bytes::<(String, char, i64)>(input).unwrap(),
            (String::from("a"), 'b', 1i64)
        );
    }
}
