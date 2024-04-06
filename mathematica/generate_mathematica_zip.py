#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Mathematica ZIP Generator
Copyright (C) 2012-2015, 2017-2018 Matthias Bolte <matthias@tinkerforge.com>
Copyright (C) 2011 Olaf Lüke <olaf@tinkerforge.com>
Copyright (C) 2020 Erik Fleckstein <erik@tinkerforge.com>

generate_mathematica_zip.py: Generator for Mathematica ZIP

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public
License along with this program; if not, write to the
Free Software Foundation, Inc., 59 Temple Place - Suite 330,
Boston, MA 02111-1307, USA.
"""

import sys

if sys.hexversion < 0x3040000:
    print('Python >= 3.4 required')
    sys.exit(1)

import os
import shutil
import importlib.util
import importlib.machinery

def create_generators_module():
    generators_dir = os.path.split(os.path.dirname(os.path.realpath(__file__)))[0]

    if sys.hexversion < 0x3050000:
        generators_module = importlib.machinery.SourceFileLoader('generators', os.path.join(generators_dir, '__init__.py')).load_module()
    else:
        generators_spec = importlib.util.spec_from_file_location('generators', os.path.join(generators_dir, '__init__.py'))
        generators_module = importlib.util.module_from_spec(generators_spec)

        generators_spec.loader.exec_module(generators_module)

    sys.modules['generators'] = generators_module

if 'generators' not in sys.modules:
    create_generators_module()

from generators import common
from generators.mathematica import mathematica_common

class MathematicaZipGenerator(mathematica_common.MathematicaGeneratorTrait, common.ZipGenerator):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.tmp_dir                    = self.get_zip_dir()
        self.tmp_source_tinkerforge_dir = os.path.join(self.tmp_dir, 'source', 'Tinkerforge')
        self.tmp_examples_dir           = os.path.join(self.tmp_dir, 'examples')

    def prepare(self):
        super().prepare()

        os.makedirs(self.tmp_source_tinkerforge_dir)
        os.makedirs(self.tmp_examples_dir)

    def generate(self, device):
        if not device.is_released():
            return

        # Copy device examples
        tmp_examples_device_dir = os.path.join(self.tmp_examples_dir,
                                               device.get_category().camel,
                                               device.get_name().camel)

        if not os.path.exists(tmp_examples_device_dir):
            os.makedirs(tmp_examples_device_dir)

        for example in common.find_device_examples(device, r'^Example.*\.nb$'):
            shutil.copy(example[1], tmp_examples_device_dir)

    def finish(self):
        root_dir = self.get_root_dir()

        # Copy IP Connection examples
        if self.get_config_name().space == 'Tinkerforge':
            for example in common.find_examples(root_dir, r'^Example.*\.nb$'):
                shutil.copy(example[1], self.tmp_examples_dir)

        # Copy bindings and readme
        for filename in self.get_released_files():
            shutil.copy(os.path.join(self.get_bindings_dir(), filename), self.tmp_source_tinkerforge_dir)

        shutil.copy(os.path.join(root_dir, '..', 'csharp', 'IPConnection.cs'), self.tmp_source_tinkerforge_dir)
        shutil.copy(os.path.join(root_dir, 'changelog.txt'),                   self.tmp_dir)
        shutil.copy(os.path.join(root_dir, 'readme.txt'),                      self.tmp_dir)
        shutil.copy(os.path.join(root_dir, '..', 'configs', 'license.txt'),    self.tmp_dir)

        # Make AssemblyInfo.cs
        version = self.get_changelog_version()

        common.specialize_template(os.path.join(root_dir, '..', 'csharp', 'AssemblyInfo.cs.template'),
                                   os.path.join(self.tmp_source_tinkerforge_dir, 'AssemblyInfo.cs'),
                                   {'<<BINDINGS>>': 'Mathematica',
                                    '<<VERSION>>': '.'.join(version)})

        # Make Tinkerforge.csproj
        project_items = []

        for filename in ['AssemblyInfo.cs', 'IPConnection.cs'] + self.get_released_files():
            project_items.append('<Compile Include="{0}" />'.format(filename))

        common.specialize_template(os.path.join(root_dir, '..', 'csharp', 'Tinkerforge.csproj.template'),
                                   os.path.join(self.tmp_source_tinkerforge_dir, 'Tinkerforge.csproj'),
                                   {'{{ITEMS}}': '\n    '.join(project_items)})

        # Make dll
        with common.ChangedDirectory(self.tmp_source_tinkerforge_dir):
            common.execute(['dotnet', 'build', '-c', 'Release', '-f', 'net20', '-o', self.tmp_dir])

        os.remove(os.path.join(self.tmp_dir, 'Tinkerforge.pdb'))
        os.remove(os.path.join(self.tmp_dir, 'Tinkerforge.xml'))
        shutil.rmtree(os.path.join(self.tmp_source_tinkerforge_dir, 'bin'))
        shutil.rmtree(os.path.join(self.tmp_source_tinkerforge_dir, 'obj'))

        # Make zip
        self.create_zip_file(self.tmp_dir)

def generate(root_dir, language, internal):
    common.generate(root_dir, language, internal, MathematicaZipGenerator)

if __name__ == '__main__':
    args = common.dockerize('mathematica', __file__, add_internal_argument=True)

    generate(os.getcwd(), 'en', args.internal)
