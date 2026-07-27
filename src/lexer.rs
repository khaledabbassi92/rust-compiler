use crate::core::{TokenType, Text};


pub fn scan(text: &mut Text, tokens: &mut Vec<TokenType>){
	while text.current < text.length {
		
		let bytes = text.source.as_bytes();
		let c = bytes[text.current as usize] as char;
		
		match c {
		'(' => {tokens.push(TokenType::LEFTPAR);
				text.current += 1;
				}
		')' => {tokens.push(TokenType::RIGHTPAR);
				text.current += 1;
				}
		'=' => {tokens.push(TokenType::EQ);
				text.current += 1;
				}
		'+' => {tokens.push(TokenType::PLUS);
				text.current += 1;
				}
		'-' => {tokens.push(TokenType::MINUS);
				text.current += 1;
				}
		';' => {tokens.push(TokenType::SEMICOLON);
				text.current += 1;
				}
		c if c.is_alphabetic()=> {
				let mut word = String::new();
				word.push(c);
				text.current += 1;
				while text.current < text.length {
					
					let c = text.source.as_bytes()[text.current] as char;
					if c.is_alphanumeric() {
						word.push(c);
						text.current += 1;
						}
						else{
							break;
						}
							}
					match word.as_str(){
						
						"let" | "fn" | "if" => tokens.push(TokenType::KEYWORD(word.clone())),
						_ => tokens.push(TokenType::IDENTIFIER(word.clone())),
						
						}
					
					

				
					}
		n if n.is_numeric() => {
			let mut number = String::new();
			number.push(n);
			text.current += 1;
			
			while text.current < text.length{
				
				let n = text.source.as_bytes()[text.current] as char;
				if n.is_numeric(){
					number.push(n);
					text.current += 1;
				}
				else{
					break;
				}
				
			}
			
			tokens.push(TokenType::NUMBER(number.clone()));
		}
			
		' ' | '\t' | '\n' | '\r' => text.current += 1,
		
		_ => text.current += 1,
			
			}
		}
		
		
		}
		

