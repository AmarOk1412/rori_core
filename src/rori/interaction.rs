/**
 * Copyright (c) 2018, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/

use rori::user::Device;
use std::collections::HashMap;
use serde::ser::{Serialize, SerializeStruct};
use serde::Serializer;
use std::fmt;
use time::Tm;

/**
 * Represent a RING interaction, just here to store informations.
 * NOTE: need a type attribute in the future.
 **/
#[derive(Clone)]
pub struct Interaction
{
    pub device_author: Device,
    pub body: String,
    pub datatype: String,
    pub metadatas: HashMap<String, String>,
    pub time: Tm
}

// Used for println!
impl fmt::Display for Interaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({};{:?}): {}",
            self.device_author, self.datatype, self.metadatas, self.body
        )
    }
}

/**
 * Used for serde_json
 */
impl Serialize for Interaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        // 5 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Interaction", 5)?;
        state.serialize_field("device_author", &self.device_author).unwrap();
        state.serialize_field("body", &self.body).unwrap();
        state.serialize_field("metadatas", &self.metadatas).unwrap();
        state.serialize_field("body", &self.body).unwrap();
        state.serialize_field("time", &self.time.rfc3339().to_string()).unwrap();
        state.end()
    }
}
