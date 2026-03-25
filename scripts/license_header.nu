#!/usr/bin/env nu
###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###

let source_root = $env.FILE_PWD

let rs_license_notice = "/*
 * Copyright (c) SimChopper; Jovan Gerodetti and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */" | lines

let rs_license_length = $rs_license_notice | length

let py_license_notice = "###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###" | lines

let py_license_length = $py_license_notice | length

let nu_license_notice = "#!/usr/bin/env nu
###
# Copyright (c) SimChopper; Jovan Gerodetti and contributors.
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
###" | lines

let nu_license_length = $nu_license_notice | length

def read_file [path: string]: nothing -> list<string> {
  open --raw $path | lines
}

def lines []: string -> list<string> {
  split row "\n"
}

def main []: nothing -> nothing {
  # rust sources
  for file in (glob $"($source_root)/**/*.{rs}") {
    let current_header = read_file $file | first $rs_license_length
   
    if $current_header == $rs_license_notice {
      continue
    }

    read_file $file | prepend ($rs_license_notice | append "") | str join "\n" | collect | save -f $file
  }

  # python and gd script sources
  for file in (glob $"($source_root)/{src,blender,.github}/**/*.{py,gd,yml}") {
    if ($file | str contains "godot-msgpack") {
      continue
    }
  
    let current_header = read_file $file | first $py_license_length
   
    if $current_header == $py_license_notice {
      continue
    }

    read_file $file | prepend ($py_license_notice | append "") | str join "\n" | collect | save -f $file
  }

  # nushell sources
  for file in (glob $"($source_root)/**/*.{nu}") {
    let current_header = read_file $file | first $nu_license_length
   
    if $current_header == $nu_license_notice {
      continue
    }

    read_file $file | prepend ($nu_license_notice | append "") | str join "\n" | collect | save -f $file
  }
}
