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

/**
 * Represent a RORI device
 * A device for RORI is a ring_id. This device can be linked to a specific user or to the anonymous
 */
#[derive(Debug, Clone)]
pub struct Device {
    pub name: String,
    pub ring_id: String
}

impl Device {
    /**
     * Generate a new device for the given ring_id
     * @param ring_id
     */
    pub fn new(ring_id: &String) -> Device {
        Device {
            name: String::new(),
            ring_id: ring_id.clone()
        }
    }
}

/**
 * Represent a RORI User
 * A user for RORI is someone who uses the system under an identity. For example, Alice
 * (real person) can uses her phone, her computer linked to RORI under the identity alice@rori
 * (user for RORI) and an IRC client as an anonymous user.
 */
#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub devices: Vec<Device>
}

impl User {
    /**
     * Generate a new anonymous user
     */
    pub fn new() -> User {
        User {
            name: String::new(),
            devices: Vec::new()
        }
    }
}
