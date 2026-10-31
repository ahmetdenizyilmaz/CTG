//! Read numeric array literals from `ZzFXM` exports without evaluating `JavaScript`.
use serde_json::Value;

pub(super) fn assignment(text: &str, name: &str) -> Result<Value, String> {
    let marker = format!("const {name} =");
    let text = text
        .split_once(&marker)
        .ok_or_else(|| format!("missing {name}"))?
        .1;
    let mut parser = Parser {
        text: text.as_bytes(),
        at: 0,
    };
    parser.value(0)
}
struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        loop {
            while self.text.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
            if self.text.get(self.at..self.at + 2) == Some(b"//") {
                while self.text.get(self.at).is_some_and(|b| *b != b'\n') {
                    self.at += 1;
                }
            } else {
                break;
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > 8 {
            return Err("array nesting too deep".into());
        }
        self.space();
        if self.text.get(self.at) == Some(&b'[') {
            self.at += 1;
            let mut values = Vec::new();
            loop {
                self.space();
                if self.text.get(self.at) == Some(&b']') {
                    self.at += 1;
                    return Ok(Value::Array(values));
                }
                if self.text.get(self.at) == Some(&b',') {
                    values.push(Value::Null);
                    self.at += 1;
                    continue;
                }
                values.push(self.value(depth + 1)?);
                self.space();
                match self.text.get(self.at) {
                    Some(b',') => self.at += 1,
                    Some(b']') => {}
                    _ => return Err("expected comma or array end".into()),
                }
            }
        }
        if self.text.get(self.at..self.at + 4) == Some(b"null") {
            self.at += 4;
            return Ok(Value::Null);
        }
        let start = self.at;
        while self
            .text
            .get(self.at)
            .is_some_and(|b| b.is_ascii_digit() || b"+-.eE".contains(b))
        {
            self.at += 1;
        }
        if start == self.at {
            return Err("only literal numeric song data can be imported".into());
        }
        let value = std::str::from_utf8(&self.text[start..self.at])
            .map_err(|e| e.to_string())?
            .parse::<f64>()
            .map_err(|e| e.to_string())?;
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| "non-finite number".into())
    }
}
