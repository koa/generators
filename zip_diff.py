#!/usr/bin/env python
# -*- coding: utf-8 -*-

import sys

if sys.hexversion < 0x3040000:
    print('Python >= 3.4 required')
    sys.exit(1)

import os
import re
import tempfile
import importlib.util
import importlib.machinery

generators_dir = os.path.dirname(os.path.realpath(__file__))

def create_generators_module():
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

args = sys.argv[1:]

for path in [os.path.expanduser('~/.zip_diffrc'), './.zip_diffrc']:
    if os.path.exists(path):
        with open(path) as f:
            args += f.readline().replace('\n', '').split(' ')

diff_tool = 'geany'

try:
    diff_tool_idx = args.index('--diff-tool')
    diff_tool = args[diff_tool_idx + 1]
    args = args[:diff_tool_idx] + args[diff_tool_idx + 2:]
except:
    pass

if len(args) == 0:
    bindings = os.path.split(os.getcwd())[-1]
else:
    bindings = args[0].rstrip('/')

root = os.path.split(__file__)[0]

if len(root) == 0:
    root = '.'

with common.ChangedDirectory(root):
    version = common.get_changelog_version(bindings)

base = os.path.join(root, bindings)
tmp = tempfile.mkdtemp()

if os.system('bash -cex "curl https://download.tinkerforge.com/bindings/{0}/tinkerforge_{0}_bindings_latest.zip -o {1}/tinkerforge_{0}_bindings_latest.zip"'.format(bindings, tmp)) != 0:
    print('download latest.zip failed')
    sys.exit(1)

if os.system('bash -cex "pushd {1} && unzip -q -d latest tinkerforge_{0}_bindings_latest.zip && popd"'.format(bindings, tmp)) != 0:
    print('unzip latest.zip failed')
    sys.exit(1)

if os.system('bash -cex "cp {0}/tinkerforge_{1}_bindings_{3}_{4}_{5}.zip {2} && pushd {2} && unzip -q -d {3}_{4}_{5} tinkerforge_{1}_bindings_{3}_{4}_{5}.zip && popd"'.format(base, bindings, tmp, *version)) != 0:
    print('copy/unzip current.zip failed')
    sys.exit(1)

if os.system('bash -cx "pushd {0} && diff -ru6 latest/ {1}_{2}_{3}/ > diff1.diff; popd"'.format(tmp, *version)) != 0:
    print('diff latest vs current failed')
    sys.exit(1)

with open(os.path.join(tmp, 'diff1.diff'), 'r') as f:
    diffs = [[[]]] # list of diffs as lists of lines

    for line in f.readlines():
        if line.startswith('diff ') or line[0] not in ['@', '-', '+', ' ']:
            diffs.append([[]])

        if line.startswith('@@ '):
            diffs[-1].append([])

        diffs[-1][-1].append(line)

c_like_header1 = re.compile(r'^@@ -1,8 \+1,8 @@\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'  \* .+ Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n$')

c_like_header2 = re.compile(r'^@@ -1,10 \+1,10 @@\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'- \* .+ Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'\+ \* .+ Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n' + \
'  \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*/\n' + \
' \n$')

delphi_header1 = re.compile(r'^@@ -1,8 \+1,8 @@\n' + \
' {\n' + \
'-  This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.\n' + \
'\+  This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.\n' + \
' \n' + \
'   Delphi/Lazarus Bindings Version 2\.[0-9]+\.[0-9]+\n' + \
' \n' + \
'   If you have a bugfix for this file and want to commit it,\n' + \
'   please fix the bug in the generator\. You can find a link\n' + \
'   to the generators git on tinkerforge\.com\n$')

delphi_header2 = re.compile(r'^@@ -1,10 \+1,10 @@\n' + \
' {\n' + \
'-  This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.\n' + \
'\+  This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.\n' + \
' \n' + \
'-  Delphi/Lazarus Bindings Version 2\.[0-9]+\.[0-9]+\n' + \
'\+  Delphi/Lazarus Bindings Version 2\.[0-9]+\.[0-9]+\n' + \
' \n' + \
'   If you have a bugfix for this file and want to commit it,\n' + \
'   please fix the bug in the generator\. You can find a link\n' + \
'   to the generators git on tinkerforge\.com\n' + \
' }\n' + \
' \n$')

javascript_header1 = re.compile(r'^@@ -[0-9]+,8 \+[0-9]+,8 @@\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'  \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n$')

javascript_header2 = re.compile(r'^@@ -[0-9]+,10 \+[0-9]+,10 @@\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'- \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'\+ \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n' + \
'  \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*/\n' + \
' \n$')

javascript_header3 = re.compile(r'^@@ -[0-9]+,13 \+[0-9]+,13 @@\n' + \
' }\n' + \
' \n' + \
' module\.exports = Brick[A-Za-z0-9]+;\n' + \
' \n' + \
' },{"\./Device":[0-9]+,"\./IPConnection":[0-9]+}\],[0-9]+:\[function\(require,module,exports\){\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'  \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n$')

javascript_header4 = re.compile(r'^@@ -[0-9]+,15 \+[0-9]+,15 @@\n' + \
' }\n' + \
' \n' + \
' module\.exports = Brick[A-Za-z0-9]+;\n' + \
' \n' + \
' },{"\./Device":[0-9]+,"\./IPConnection":[0-9]+}\],[0-9]+:\[function\(require,module,exports\){\n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'- \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'\+ \* JavaScript Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n' + \
'  \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*/\n' + \
' \n$')

perl_header1 = re.compile(r'^@@ -1,8 \+1,8 @@\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
' # Perl Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n$')

perl_header2 = re.compile(r'^@@ -1,10 \+1,10 @@\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
'-# Perl Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
'\+# Perl Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n' + \
' #############################################################\n' + \
' \n$')

php_header1 = re.compile(r'^@@ -1,10 \+1,10 @@\n' + \
' <\?php\n' + \
' \n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'  \* PHP Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n$')

php_header2 = re.compile(r'^@@ -1,12 \+1,12 @@\n' + \
' <\?php\n' + \
' \n' + \
' /\* \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\\n' + \
'- \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'\+ \* This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      \*\n' + \
'  \*                                                           \*\n' + \
'- \* PHP Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'\+ \* PHP Bindings Version 2\.[0-9]+\.[0-9]+[ ]+\*\n' + \
'  \*                                                           \*\n' + \
'  \* If you have a bugfix for this file and want to commit it, \*\n' + \
'  \* please fix the bug in the generator\. You can find a link  \*\n' + \
'  \* to the generators git repository on tinkerforge\.com       \*\n' + \
'  \*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*\*/\n' + \
' \n$')

python_header1 = re.compile(r'^@@ -1,9 \+1,9 @@\n' + \
' # -\*- coding: utf-8 -\*-\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
' # Python Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n$')

python_header2 = re.compile(r'^@@ -1,11 \+1,11 @@\n' + \
' # -\*- coding: utf-8 -\*-\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
'-# Python Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
'\+# Python Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n' + \
' #############################################################\n' + \
' \n$')

ruby_header1 = re.compile(r'^@@ -1,9 \+1,9 @@\n' + \
' # -\*- ruby encoding: utf-8 -\*-\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
' # Ruby Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n$')

ruby_header2 = re.compile(r'^@@ -1,11 \+1,11 @@\n' + \
' # -\*- ruby encoding: utf-8 -\*-\n' + \
' #############################################################\n' + \
'-# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
'\+# This file was automatically generated on [0-9]{4}-[0-9]{2}-[0-9]{2}\.      #\n' + \
' #                                                           #\n' + \
'-# Ruby Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
'\+# Ruby Bindings Version 2\.[0-9]+\.[0-9]+[ ]+#\n' + \
' #                                                           #\n' + \
' # If you have a bugfix for this file and want to commit it, #\n' + \
' # please fix the bug in the generator\. You can find a link  #\n' + \
' # to the generators git repository on tinkerforge\.com       #\n' + \
' #############################################################\n' + \
' \n$')

filtered = []

for diff in diffs:
    filtered_lines = []

    for lines in diff:
        if len(lines) == 0:
            continue

        hunk = ''.join(lines)

        if not c_like_header1.match(hunk) and \
           not c_like_header2.match(hunk) and \
           not delphi_header1.match(hunk) and \
           not delphi_header2.match(hunk) and \
           not javascript_header1.match(hunk) and \
           not javascript_header2.match(hunk) and \
           not javascript_header3.match(hunk) and \
           not javascript_header4.match(hunk) and \
           not perl_header1.match(hunk) and \
           not perl_header2.match(hunk) and \
           not php_header1.match(hunk) and \
           not php_header2.match(hunk) and \
           not python_header1.match(hunk) and \
           not python_header2.match(hunk) and \
           not ruby_header1.match(hunk) and \
           not ruby_header2.match(hunk):
            filtered_lines += lines
        else:
            filtered_lines += [lines[0].rstrip() + ' // dropped header hunk\n']

    if len(filtered_lines) == 0:
        continue

    if len(filtered_lines) == 4 and \
       filtered_lines[0].startswith('diff -ru6 ') and \
       filtered_lines[1].startswith('--- ') and \
       filtered_lines[2].startswith('+++ ') and \
       filtered_lines[3].endswith('// dropped header hunk\n'):
        filtered += [filtered_lines[0].rstrip() + ' // dropped header diff\n']
    else:
        filtered += filtered_lines

with open(os.path.join(tmp, 'diff2.diff'), 'w') as f:
    f.writelines(filtered)

if os.system('bash -c "pushd {} && {} diff2.diff && popd"'.format(tmp, diff_tool)) != 0:
    print('{} diff.diff failed'.format(diff_tool))
    sys.exit(1)
