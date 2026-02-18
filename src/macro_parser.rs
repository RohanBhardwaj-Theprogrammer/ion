// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Rohan Bhardwaj
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashSet ;


/// A simple macro parser that can parse #define , #ifdef, #ifndef, #undef directives from a given file content
/// 
struct MacroParser {
    def_macros: HashSet<String>,
    undef_macros: HashSet<String>,

}

impl MacroParser {
    pub fn new() -> Self {
        MacroParser {
            def_macros: HashSet::new(),
            undef_macros: HashSet::new(),
        }
    }

    pub fn parse(&mut self, content:&str)-> Vec<String> {

        let line_iter = content.lines() ;
        
        let mut headers = Vec::new() ;
        self.parse_directives(line_iter(), &mut headers) ;
        return headers ;
    }
    
    fn parse_directives(&mut self,line :&std::Iter::Line ,headers:&mut Vec<String>) -> Vec<String> {

     
       while let Some(line) = line_iter.next() {
            let directives ; 
            match self.parse_line(line) {
                Some(directives_vec) => directives = directives_vec,
                None => continue,
            }


            match directives[0] {
                "endif" | "else" => {
                    return ;
                }
                "include" => {
                    if directives.len() > 1 {
                        let header = directives[1].trim_matches(|c| c == '"' || c == '<' || c == '>');

                        #[cfg(any(test,debug_assertions))]
                        println!(" Found header : {} ", header);

                        headers.push(header.to_string());
                    }
                }

                "define" => {
                    if directives.len() > 1 {
                        self.def_macros.insert(directives[1].to_string());
                        self.undef_macros.remove(directives[1]);
                    }
                }
                "undef" => {
                    if directives.len() > 1 {
                        self.undef_macros.insert(directives[1].to_string());
                        def_macros.remove(directives[1]);
                    }
                }
               
               "ifdef" => {
                    if directives.len() > 1 {
                        let macro_name = directives[1];
                        if self.def_macros.contains(macro_name) {
                           
                            parse_directives(line_iter, headers,) ;
                        } else {
                            while let some(next_line) = line_iter.peek() {
                                if is_directive(next_line) {
                                  let next_directives = self.parse_line(next_line) ;
                                    if next_directives[0] == "endif" || next_directives[0] == "else" {
                                        break ;
                                    }
                                }
                                line_iter.next() ;
                            } 
                        }
                    }
                }
                "ifndef" => {
                    if directives.len() > 1 {
                        let macro_name = directives[1];
                        if !self.def_macros.contains(macro_name) {
                            parse_directives(line_iter, headers,) ;
                        } else {
                             while let some(next_line) = line_iter.peek() {
                                if is_directive(next_line) {
                                  let next_directives = self.parse_line(next_line) ;
                                    if next_directives[0] == "endif" || next_directives[0] == "else" {
                                        break ;
                                    }
                                }
                                line_iter.next() ;
                            } 
                        } 
                    }
               }
               
            }
        }
    }

    fn parse_line(&self, line:&str) ->Option<Vec<&str>>{
        line.trim_start(); 
        if !line.starts_with('#') {
            return None ;
        }
        let directives =  line[1..].split_whitespaces().collect::<Vec<&str>>()[0] ;
        if directives.is_empty() {
            return None ;
        }
        return Some(directives) ;
    }
    fn is_directive(&self, line:&str) -> bool {
        line.trim_start().starts_with('#')
    }
}