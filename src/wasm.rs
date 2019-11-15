use std::str;
use string_strategy::{AsciiStringStrategy, UnicodeiStringStrategy};
use symspell::{SymSpell, Verbosity};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct JSSuggestion {
    term: String,
    distance: i32,
    count: i32,
}

#[derive(Serialize, Deserialize)]
pub struct InitParams {
    is_ascii: bool,
    max_edit_distance: i32,
    prefix_length: i32,
    count_threshold: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DictParams {
    term_index: i32,
    count_index: i32,
    separator: String,
}

#[wasm_bindgen(js_name = SymSpell)]
pub struct JSSymSpell {
    symspell: EJSSymSpell,
}

pub enum EJSSymSpell {
    Ascii(SymSpell<AsciiStringStrategy>),
    Unicode(SymSpell<UnicodeiStringStrategy>),
}

#[wasm_bindgen(js_class = SymSpell)]
impl JSSymSpell {
    #[wasm_bindgen(constructor)]
    pub fn new(parameters: &JsValue) -> Result<JSSymSpell, JsValue> {
        let params: InitParams;
        let ret: JSSymSpell;

        if let Ok(i) = parameters.into_serde() {
            params = i;
        } else {
            return Err(JsValue::from("Unable to parse arguments"));
        }

        if params.is_ascii {
            ret = JSSymSpell {
                symspell: EJSSymSpell::Ascii(SymSpell::new(
                    params.max_edit_distance as i64,
                    params.prefix_length as i64,
                    params.count_threshold as i64,
                    0 as i64,
                )),
            };
        } else {
            ret = JSSymSpell {
                symspell: EJSSymSpell::Unicode(SymSpell::new(
                    params.max_edit_distance as i64,
                    params.prefix_length as i64,
                    params.count_threshold as i64,
                    0 as i64,
                )),
            };
        }
        Ok(ret)
    }

    // Expose numeric params as i32 and cast to i64 is required bc BigInt doesn'tplay well in some
    // browsers.
    pub fn load_dictionary(&mut self, input: &[u8], args: &JsValue) -> Result<(), JsValue> {
        let params: DictParams;
        if let Ok(i) = args.into_serde() {
            params = i;
        } else {
            return Err(JsValue::from("Unable to parse arguments"));
        }

        let corpus: &str;
        if let Ok(i) = str::from_utf8(input) {
            corpus = i;
        } else {
            return Err(JsValue::from("Invalid UTF-8"));
        }

        for line in corpus.lines() {
            match self.symspell {
                EJSSymSpell::Ascii(ref mut i) => i.load_dictionary_line(
                    &line,
                    params.term_index as i64,
                    params.count_index as i64,
                    &params.separator,
                ),
                EJSSymSpell::Unicode(ref mut i) => i.load_dictionary_line(
                    &line,
                    params.term_index as i64,
                    params.count_index as i64,
                    &params.separator,
                ),
            };
        }
        Ok(())
    }

    pub fn lookup_compound(
        &self,
        input: &str,
        edit_distance: i32,
    ) -> Result<Vec<JsValue>, JsValue> {
        let res = match self.symspell {
            EJSSymSpell::Ascii(ref i) => i.lookup_compound(input, edit_distance as i64),
            EJSSymSpell::Unicode(ref i) => i.lookup_compound(input, edit_distance as i64),
        };
        Ok(res
            .into_iter()
            .map(|sugg| {
                let temp = JSSuggestion {
                    term: sugg.term,
                    distance: sugg.distance as i32,
                    count: sugg.count as i32,
                };
                JsValue::from_serde(&temp).unwrap()
            })
            .collect())
    }

    pub fn lookup(
        &self,
        input: &str,
        verbosity: i8,
        max_edit_distance: i32,
    ) -> Result<Vec<JsValue>, JsValue> {
        let sym_verbosity = match verbosity {
            0 => Verbosity::Top,
            1 => Verbosity::All,
            2 => Verbosity::Closest,
            _ => return Err(JsValue::from("Verbosity must be between 0 and 2")),
        };

        let res = match self.symspell {
            EJSSymSpell::Ascii(ref i) => i.lookup(input, sym_verbosity, max_edit_distance as i64),
            EJSSymSpell::Unicode(ref i) => {
                i.lookup(&input, sym_verbosity, max_edit_distance as i64)
            }
        };

        Ok(res
            .into_iter()
            .map(|sugg| {
                let temp = JSSuggestion {
                    term: sugg.term,
                    distance: sugg.distance as i32,
                    count: sugg.count as i32,
                };
                JsValue::from_serde(&temp).unwrap()
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_sentence() {
        let init_args = InitParams {
            is_ascii: false,
            max_edit_distance: 2,
            prefix_length: 7,
            count_threshold: 1,
        };
        let mut speller = JSSymSpell::new(&JsValue::from_serde(&init_args).unwrap()).unwrap();
        let sentence = "wher";
        let dict = "where 360468339\ninfo 352363058".as_bytes();
        let expected = "where";

        let dict_args = DictParams {
            term_index: 0,
            count_index: 1,
            separator: String::from(" "),
        };
        speller
            .load_dictionary(dict, &JsValue::from_serde(&dict_args).unwrap())
            .unwrap();
        let result: JSSuggestion = speller.lookup_compound(sentence, 1).unwrap()[0]
            .into_serde()
            .unwrap();
        assert_eq!(result.term, expected);
    }
}
