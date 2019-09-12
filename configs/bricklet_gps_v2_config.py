# -*- coding: utf-8 -*-

# Redistribution and use in source and binary forms of this file,
# with or without modification, are permitted. See the Creative
# Commons Zero (CC0 1.0) License for more details.

# GPS Bricklet 2.0 communication config

from openhab_common import *

com = {
    'author': 'Olaf Lüke <olaf@tinkerforge.com>',
    'api_version': [2, 0, 1],
    'category': 'Bricklet',
    'device_identifier': 276,
    'name': 'GPS V2',
    'display_name': 'GPS 2.0',
    'manufacturer': 'Tinkerforge',
    'description': {
        'en': 'Determine position, velocity and altitude using GPS',
        'de': 'Bestimmt Position, Geschwindigkeit und Höhe mittels GPS'
    },
    'released': True,
    'documented': True,
    'discontinued': False,
    'features': [
        'comcu_bricklet',
        'bricklet_get_identity'
    ],
    'constant_groups': [],
    'packets': [],
    'examples': []
}

com['constant_groups'].append({
'name': 'Restart Type',
'type': 'uint8',
'constants': [('Hot Start', 0),
              ('Warm Start', 1),
              ('Cold Start', 2),
              ('Factory Reset', 3)]
})

com['constant_groups'].append({
'name': 'Satellite System',
'type': 'uint8',
'constants': [('GPS', 0),
              ('GLONASS', 1),
              ('Galileo', 2)]
})

com['constant_groups'].append({
'name': 'Fix',
'type': 'uint8',
'constants': [('No Fix', 1),
              ('2D Fix', 2),
              ('3D Fix', 3)]
})

com['constant_groups'].append({
'name': 'Fix LED Config',
'type': 'uint8',
'constants': [('Off', 0),
              ('On', 1),
              ('Show Heartbeat', 2),
              ('Show Fix', 3),
              ('Show PPS', 4)]
})

com['constant_groups'].append({
'name': 'SBAS',
'type': 'uint8',
'constants': [('Enabled', 0),
              ('Disabled', 1)]
})

com['packets'].append({
'type': 'function',
'name': 'Get Coordinates',
'elements': [('Latitude', 'uint32', 1, 'out'),
             ('NS', 'char', 1, 'out'),
             ('Longitude', 'uint32', 1, 'out'),
             ('EW', 'char', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the GPS coordinates. Latitude and longitude are given in the
``DD.dddddd°`` format, the value 57123468 means 57.123468°.
The parameter ``ns`` and ``ew`` are the cardinal directions for
latitude and longitude. Possible values for ``ns`` and ``ew`` are 'N', 'S', 'E'
and 'W' (north, south, east and west).

This data is only valid if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Gibt die GPS Koordinaten zurück. Breitengrad und Längengrad werden im Format
``DD.dddddd°`` angegeben, der Wert 57123468 bedeutet 57,123468°.
Die Parameter ``ns`` und ``ew`` sind Himmelsrichtungen für
Breiten- und Längengrad. Mögliche Werte für ``ns`` und ``ew`` sind 'N', 'S', 'E'
und 'W' (Nord, Süd, Ost, West).

Diese Daten sind nur gültig wenn ein Fix vorhanden ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Status',
'elements': [('Has Fix', 'bool', 1, 'out'),
             ('Satellites View', 'uint8', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns if a fix is currently available as well as the, the number of
satellites that are in view.

There is also a :ref:`green LED <gps_v2_bricklet_fix_led>` on the Bricklet that
indicates the fix status.
""",
'de':
"""
Gibt zurück ob ein GPS Fix besteht sowie die Anzahl der sichtbaren Satelliten.

Auf dem Bricklet ist eine :ref:`green LED <gps_v2_bricklet_fix_led>`, die den
Fix-Status anzeigt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Altitude',
'elements': [('Altitude', 'int32', 1, 'out'),
             ('Geoidal Separation', 'int32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the current altitude and corresponding geoidal separation.

Both values are given in cm.

This data is only valid if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Gibt die aktuelle Höhe und die dazu gehörige Geoidal Separation zurück.

Beide Werte werden in cm angegeben.

Diese Daten sind nur gültig wenn ein Fix vorhanden ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Motion',
'elements': [('Course', 'uint32', 1, 'out'),
             ('Speed', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the current course and speed. Course is given in hundredths degree
and speed is given in hundredths km/h. A course of 0° means the Bricklet is
traveling north bound and 90° means it is traveling east bound.

Please note that this only returns useful values if an actual movement
is present.

This data is only valid if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Gibt die aktuelle Richtung und Geschwindigkeit zurück. Die Richtung wird
in hundertstel Grad und die Geschwindigkeit in hundertstel km/h angegeben. Eine
Richtung von 0° bedeutet eine Bewegung des Bricklets nach Norden und 90°
einer Bewegung nach Osten.

Dabei ist zu beachten: Diese Funktion liefert nur nützlich Werte wenn
auch tatsächlich eine Bewegung stattfindet.

Diese Daten sind nur gültig wenn ein Fix vorhanden ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Date Time',
'elements': [('Date', 'uint32', 1, 'out'),
             ('Time', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the current date and time. The date is
given in the format ``ddmmyy`` and the time is given
in the format ``hhmmss.sss``. For example, 140713 means
14.07.13 as date and 195923568 means 19:59:23.568 as time.
""",
'de':
"""
Gibt das aktuelle Datum und die aktuelle Zeit zurück. Das Datum ist
im Format ``ddmmyy`` und die Zeit im Format ``hhmmss.sss`` angegeben. Zum
Beispiel, 140713 bedeutet 14.07.13 als Datum und 195923568 bedeutet
19:59:23.568 als Zeit.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Restart',
'elements': [('Restart Type', 'uint8', 1, 'in', {'constant_group': 'Restart Type'})],
'since_firmware': [1, 0, 0],
'doc': ['af', {
'en':
"""
Restarts the GPS Bricklet, the following restart types are available:

.. csv-table::
 :header: "Value", "Description"
 :widths: 10, 100

 "0", "Hot start (use all available data in the NV store)"
 "1", "Warm start (don't use ephemeris at restart)"
 "2", "Cold start (don't use time, position, almanacs and ephemeris at restart)"
 "3", "Factory reset (clear all system/user configurations at restart)"
""",
'de':
"""
Startet das GPS Bricklet neu. Die folgenden Neustart-Typen stehen zur
Verfügung:

.. csv-table::
 :header: "Wert", "Beschreibung"
 :widths: 10, 100

 "0", "Hot Start (alle verfügbaren Daten im NV-Speicher werden weiter genutzt)"
 "1", "Warm Start (Ephemerisdaten werden verworfen)"
 "2", "Cold Start (Zeit-, Position-, Almanach- und Ephemerisdaten werden verworfen)"
 "3", "Factory Reset (Alle System/User Einstellungen werden verworfen)"
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Satellite System Status Low Level',
'elements': [('Satellite System', 'uint8', 1, 'in', {'constant_group': 'Satellite System'}),
             ('Satellite Numbers Length', 'uint8', 1, 'out'),
             ('Satellite Numbers Data', 'uint8', 12, 'out'),
             ('Fix', 'uint8', 1, 'out', {'constant_group': 'Fix'}),
             ('PDOP', 'uint16', 1, 'out'),
             ('HDOP', 'uint16', 1, 'out'),
             ('VDOP', 'uint16', 1, 'out')],
'high_level': {'stream_out': {'name': 'Satellite Numbers', 'single_chunk': True}},
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the

* satellite numbers list (up to 12 items)
* fix value,
* PDOP value,
* HDOP value and
* VDOP value

for a given satellite system. Currently GPS and GLONASS are supported, Galileo
is not yet supported.

The GPS and GLONASS satellites have unique numbers and the satellite list gives
the numbers of the satellites that are currently utilized. The number 0 is not
a valid satellite number and can be ignored in the list.
""",
'de':
"""
Gibt die

* Liste der Satellitennummern (bis zu 12 Elemente),
* Fix-Wert,
* PDOP-Wert,
* HDOP-Wert and
* VDOP-Wert zurück.

für ein gegebenes Satellitensystem zurück. Aktuell werden GPS und GLONASS
unterstützt, Galileo hat noch keine Unterstützung.

Die GPS und GLONASS Satelliten haben eindeutige Nummern and die Satellitenliste
gibt die Nummer der Satelliten die aktuell benutzt werden. Die Nummer 0 ist
keine gültige Satellitennummer und kann ignoriert werden.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Satellite Status',
'elements': [('Satellite System', 'uint8', 1, 'in', {'constant_group': 'Satellite System'}),
             ('Satellite Number', 'uint8', 1, 'in'), # 1-32, for GLONASS 1-32 correspond to satellite 65-96
             ('Elevation', 'int16', 1, 'out'),       # 0-90°
             ('Azimuth', 'int16', 1, 'out'),         # 0-359°
             ('SNR', 'int16', 1, 'out')],            # 0-99dB
'since_firmware': [1, 0, 0],
'doc': ['bf', {
'en':
"""
Returns the current

* elevation (0° - 90°),
* azimuth (0° - 359°) and
* SNR (0dB - 99dB)

for a given satellite and satellite system.

The satellite number here always goes from 1 to 32. For GLONASS it corresponds to
the satellites 65-96.

Galileo is not yet supported.
""",
'de':
"""
Gibt die aktuellen Werte von

* Elevation (0° - 90°),
* Azimutwinkel (0° - 359°) und
* SNR (0dB - 99dB)

für einen gegebenen Satelliten und ein gegebenes Satellitensystem zurück.

Die Satellitennummer hat hier immer einen Bereich von 1 bis 32. Bei GLONASS
entspricht dieser Bereich den Satelliten 65-96.

Galileo wird noch nicht unterstützt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Fix LED Config',
'elements': [('Config', 'uint8', 1, 'in', {'constant_group': 'Fix LED Config'})],
'since_firmware': [1, 0, 0],
'doc': ['af', {
'en':
"""
Sets the fix LED configuration. By default the LED shows if
the Bricklet got a GPS fix yet. If a fix is established the LED turns on.
If there is no fix then the LED is turned off.

You can also turn the LED permanently on/off, show a heartbeat or let it blink
in sync with the PPS (pulse per second) output of the GPS module.

If the Bricklet is in bootloader mode, the LED is off.
""",
'de':
"""
Setzt die Konfiguration der Fix-LED. Standardmäßig zeigt
die LED an ob ein GPS-Fix besteht. Wenn ein Fix da ist, geht die LED an. Wenn
kein Fix da ist, geht die LED aus.

Die LED kann auch permanent an/aus gestellt werden, einen Herzschlag anzeigen
oder im Rythmus des PPS (Puls pro Sekunde) Ausgangs des GPS Moduls blinken.

Wenn das Bricklet sich im Bootlodermodus befindet ist die LED aus.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Fix LED Config',
'elements': [('Config', 'uint8', 1, 'out', {'constant_group': 'Fix LED Config'})],
'since_firmware': [1, 0, 0],
'doc': ['af', {
'en':
"""
Returns the configuration as set by :func:`Set Fix LED Config`
""",
'de':
"""
Gibt die Konfiguration zurück, wie von :func:`Set Fix LED Config` gesetzt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Coordinates Callback Period',
'elements': [('Period', 'uint32', 1, 'in')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Sets the period in ms with which the :cb:`Coordinates` callback is triggered
periodically. A value of 0 turns the callback off.

The :cb:`Coordinates` callback is only triggered if the coordinates changed
since the last triggering.

The default value is 0.
""",
'de':
"""
Setzt die Periode in ms mit welcher der :cb:`Coordinates` Callback ausgelöst wird.
Ein Wert von 0 deaktiviert den Callback.

Der :cb:`Coordinates` Callback wird nur ausgelöst, wenn sich die Koordinaten seit der
letzten Auslösung geändert haben.

Der Standardwert ist 0.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Coordinates Callback Period',
'elements': [('Period', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Returns the period as set by :func:`Set Coordinates Callback Period`.
""",
'de':
"""
Gibt die Periode zurück, wie von :func:`Set Coordinates Callback Period`
gesetzt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Status Callback Period',
'elements': [('Period', 'uint32', 1, 'in')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Sets the period in ms with which the :cb:`Status` callback is triggered
periodically. A value of 0 turns the callback off.

The :cb:`Status` callback is only triggered if the status changed since the
last triggering.

The default value is 0.
""",
'de':
"""
Setzt die Periode in ms mit welcher der :cb:`Status` Callback ausgelöst wird.
Ein Wert von 0 deaktiviert den Callback.

Der :cb:`Status` Callback wird nur ausgelöst, wenn sich der Status seit der
letzten Auslösung geändert hat.

Der Standardwert ist 0.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Status Callback Period',
'elements': [('Period', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Returns the period as set by :func:`Set Status Callback Period`.
""",
'de':
"""
Gibt die Periode zurück, wie von :func:`Set Status Callback Period`
gesetzt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Altitude Callback Period',
'elements': [('Period', 'uint32', 1, 'in')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Sets the period in ms with which the :cb:`Altitude` callback is triggered
periodically. A value of 0 turns the callback off.

The :cb:`Altitude` callback is only triggered if the altitude changed since the
last triggering.

The default value is 0.
""",
'de':
"""
Setzt die Periode in ms mit welcher der :cb:`Altitude` Callback ausgelöst wird.
Ein Wert von 0 deaktiviert den Callback.

Der :cb:`Altitude` Callback wird nur ausgelöst, wenn sich die Höhe seit der
letzten Auslösung geändert hat.

Der Standardwert ist 0.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Altitude Callback Period',
'elements': [('Period', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Returns the period as set by :func:`Set Altitude Callback Period`.
""",
'de':
"""
Gibt die Periode zurück, wie von :func:`Set Altitude Callback Period`
gesetzt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Motion Callback Period',
'elements': [('Period', 'uint32', 1, 'in')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Sets the period in ms with which the :cb:`Motion` callback is triggered
periodically. A value of 0 turns the callback off.

The :cb:`Motion` callback is only triggered if the motion changed since the
last triggering.

The default value is 0.
""",
'de':
"""
Setzt die Periode in ms mit welcher der :cb:`Motion` Callback ausgelöst wird.
Ein Wert von 0 deaktiviert den Callback.

Der :cb:`Motion` Callback wird nur ausgelöst, wenn sich die Bewegung seit der
letzten Auslösung geändert hat.

Der Standardwert ist 0.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Motion Callback Period',
'elements': [('Period', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Returns the period as set by :func:`Set Motion Callback Period`.
""",
'de':
"""
Gibt die Periode zurück, wie von :func:`Set Motion Callback Period`
gesetzt.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set Date Time Callback Period',
'elements': [('Period', 'uint32', 1, 'in')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Sets the period in ms with which the :cb:`Date Time` callback is triggered
periodically. A value of 0 turns the callback off.

The :cb:`Date Time` callback is only triggered if the date or time changed
since the last triggering.

The default value is 0.
""",
'de':
"""
Setzt die Periode in ms mit welcher der :cb:`Date Time` Callback ausgelöst wird.
Ein Wert von 0 deaktiviert den Callback.

Der :cb:`Date Time` Callback wird nur ausgelöst, wenn sich das Datum oder die
Zeit seit der letzten Auslösung geändert haben.

Der Standardwert ist 0.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get Date Time Callback Period',
'elements': [('Period', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['ccf', {
'en':
"""
Returns the period as set by :func:`Set Date Time Callback Period`.
""",
'de':
"""
Gibt die Periode zurück, wie von :func:`Set Date Time Callback Period`
gesetzt.
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Pulse Per Second',
'elements': [],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered precisely once per second,
see `PPS <https://en.wikipedia.org/wiki/Pulse-per-second_signal>`__.

The precision of two subsequent pulses will be skewed because
of the latency in the USB/RS485/Ethernet connection. But in the
long run this will be very precise. For example a count of
3600 pulses will take exactly 1 hour.
""",
'de':
"""
Dieser Callback wird  präzise einmal pro sekunde ausgeführt,
siehe `PPS <https://de.wikipedia.org/wiki/Puls_pro_Sekunde>`__.

Die Präzision von zwei direkt aufeinander folgenden Pulsen
kann auf Grund von Latenzen in der USB/RS485/Ethernet-Verbindung
verzerrt sein. Langfristig ist dies allerdings weiterhin sehr
präzise. Zum Beispiel wird eine Zählung von 3600 Pulsen exakt
eine Stund dauern.
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Coordinates',
'elements': [('Latitude', 'uint32', 1, 'out'),
             ('NS', 'char', 1, 'out'),
             ('Longitude', 'uint32', 1, 'out'),
             ('EW', 'char', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered periodically with the period that is set by
:func:`Set Coordinates Callback Period`. The parameters are the same
as for :func:`Get Coordinates`.

The :cb:`Coordinates` callback is only triggered if the coordinates changed
since the last triggering and if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Dieser Callback wird mit der Periode, wie gesetzt mit
:func:`Set Coordinates Callback Period`, ausgelöst. Die Parameter sind die
gleichen wie die von :func:`Get Coordinates`.

Der :cb:`Coordinates` Callback wird nur ausgelöst, wenn sich die
Koordinaten seit der letzten Auslösung geändert haben und ein Fix vorhanden
ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Status',
'elements': [('Has Fix', 'bool', 1, 'out'),
             ('Satellites View', 'uint8', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered periodically with the period that is set by
:func:`Set Status Callback Period`. The parameters are the same
as for :func:`Get Status`.

The :cb:`Status` callback is only triggered if the status changed since the
last triggering.
""",
'de':
"""
Dieser Callback wird mit der Periode, wie gesetzt mit
:func:`Set Status Callback Period`, ausgelöst. Die Parameter sind die
gleichen wie die von :func:`Get Status`.

Der :cb:`Status` Callback wird nur ausgelöst, wenn sich der
Status seit der letzten Auslösung geändert hat.
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Altitude',
'elements': [('Altitude', 'int32', 1, 'out'),
             ('Geoidal Separation', 'int32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered periodically with the period that is set by
:func:`Set Altitude Callback Period`. The parameters are the same
as for :func:`Get Altitude`.

The :cb:`Altitude` callback is only triggered if the altitude changed since the
last triggering and if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Dieser Callback wird mit der Periode, wie gesetzt mit
:func:`Set Altitude Callback Period`, ausgelöst. Die Parameter sind die
gleichen wie die von :func:`Get Altitude`.

Der :cb:`Altitude` Callback wird nur ausgelöst, wenn sich die
Höhe seit der letzten Auslösung geändert hat und ein Fix vorhanden
ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Motion',
'elements': [('Course', 'uint32', 1, 'out'),
             ('Speed', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered periodically with the period that is set by
:func:`Set Motion Callback Period`. The parameters are the same
as for :func:`Get Motion`.

The :cb:`Motion` callback is only triggered if the motion changed since the
last triggering and if there is currently a fix as indicated by
:func:`Get Status`.
""",
'de':
"""
Dieser Callback wird mit der Periode, wie gesetzt mit
:func:`Set Motion Callback Period`, ausgelöst. Die Parameter sind die
gleichen wie die von :func:`Get Motion`.

Der :cb:`Motion` Callback wird nur ausgelöst, wenn sich die
Bewegung seit der letzten Auslösung geändert hat und ein Fix vorhanden
ist (siehe :func:`Get Status`).
"""
}]
})

com['packets'].append({
'type': 'callback',
'name': 'Date Time',
'elements': [('Date', 'uint32', 1, 'out'),
             ('Time', 'uint32', 1, 'out')],
'since_firmware': [1, 0, 0],
'doc': ['c', {
'en':
"""
This callback is triggered periodically with the period that is set by
:func:`Set Date Time Callback Period`. The parameters are the same
as for :func:`Get Date Time`.

The :cb:`Date Time` callback is only triggered if the date or time changed
since the last triggering.
""",
'de':
"""
Dieser Callback wird mit der Periode, wie gesetzt mit
:func:`Set Date Time Callback Period`, ausgelöst. Die Parameter sind die
gleichen wie die von :func:`Get Date Time`.

Der :cb:`Date Time` Callback wird nur ausgelöst, wenn sich das Datum oder die
Zeit seit der letzten Auslösung geändert haben.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Set SBAS Config',
'elements': [('SBAS Config', 'uint8', 1, 'in', {'constant_group': 'SBAS'})],
'since_firmware': [2, 0, 2],
'doc': ['af', {
'en':
"""
If `SBAS <https://en.wikipedia.org/wiki/GNSS_augmentation#Satellite-based_augmentation_system>`__ is enabled,
the position accuracy increases (if SBAS satellites are in view),
but the update rate is limited to 5Hz. With SBAS disabled the update rate is increased to 10Hz.

By default SBAS is enabled and the update rate is 5Hz.
""",
'de':
"""
Wenn `SBAS <https://de.wikipedia.org/wiki/Satellite_Based_Augmentation_System>`__ aktiviert ist,
erhöht sich die Positionsgenauigkeit der GPS Daten falls SBAS Satelliten zu sehen sind.
Die Aktualisierungsrate der GPS Daten beträgt 5Hz falls SBAS aktiviert ist und 10Hz falls SBAS deaktiviert ist.

Standardmäßig ist SBAS aktiviert und die Aktualisierungsrate 5Hz.
"""
}]
})

com['packets'].append({
'type': 'function',
'name': 'Get SBAS Config',
'elements': [('SBAS Config', 'uint8', 1, 'out', {'constant_group': 'SBAS'})],

'since_firmware': [2, 0, 2],
'doc': ['af', {
'en':
"""
Returns the SBAS configuration as set by :func:`Set SBAS Config`
""",
'de':
"""
Gibt die SBAS-Konfiguration zurück, wie von :func:`Set SBAS Config` gesetzt.
"""
}]
})

com['examples'].append({
'name': 'Simple',
'functions': [('getter', ('Get Coordinates', 'coordinates'), [(('Latitude', 'Latitude'), 'uint32', 1, 1000000.0, '°', None), (('NS', 'N/S'), 'char', 1, None, None, None), (('Longitude', 'Longitude'), 'uint32', 1, 1000000.0, '°', None), (('EW', 'E/W'), 'char', 1, None, None, None)], [])]
})

com['examples'].append({
'name': 'Callback',
'functions': [('callback', ('Coordinates', 'coordinates'), [(('Latitude', 'Latitude'), 'uint32', 1, 1000000.0, '°', None), (('NS', 'N/S'), 'char', 1, None, None, None), (('Longitude', 'Longitude'), 'uint32', 1, 1000000.0, '°', None), (('EW', 'E/W'), 'char', 1, None, None, None)], None, None),
              ('callback_period', ('Coordinates', 'coordinates'), [], 1000)]
})


com['openhab'] = {
    'imports': oh_generic_channel_imports() + oh_generic_trigger_channel_imports() +
                ['org.eclipse.smarthome.core.library.types.StringType',
                 'org.eclipse.smarthome.core.library.types.PointType',
                 'org.eclipse.smarthome.core.library.types.DateTimeType',
                 'org.eclipse.smarthome.core.library.types.DecimalType',
                 'org.eclipse.smarthome.core.library.types.OnOffType',
                 'java.time.ZoneId'],
    'param_groups': oh_generic_channel_param_groups(),
    'params': [{
            'name': 'Fix LED Config',
            'type': 'integer',
            'options': [('Off', 0),
                        ('On', 1),
                        ('Show Heartbeat', 2),
                        ('Show Fix', 3),
                        ('Show PPS', 4)],
            'limitToOptions': 'true',
            'label': 'Fix LED Config',
            'description': 'The fix LED configuration. By default the LED shows if the Bricklet got a GPS fix yet. If a fix is established the LED turns on. If there is no fix then the LED is turned off.</br></br>You can also turn the LED permanently on/off, show a heartbeat or let it blink in sync with the PPS (pulse per second) output of the GPS module.<br/><br/>If the Bricklet is in bootloader mode, the LED is off.',
            'default': 3,
        }, {
            'name': 'Enable SBAS',
            'type': 'boolean',
            'default': 'true',
            'label': 'Enable SBAS',
            'description': 'If <a href=\\\"https://en.wikipedia.org/wiki/GNSS_augmentation#Satellite-based_augmentation_system\\\">SBAS</a> is enabled, the position accuracy increases (if SBAS satellites are in view), but the update rate is limited to 5Hz. With SBAS disabled the update rate is increased to 10Hz.</br></br>By default SBAS is enabled and the update rate is 5Hz.'
        }],
    'init_code': """this.setFixLEDConfig(cfg.fixLEDConfig);
    this.setSBASConfig(cfg.enableSBAS ? 0 : 1);""",
    'channels': [
        {
            'id': 'Location',
            'type': 'Location',
            'callbacks': [{
                'packet': 'Coordinates',
                'transform': "new PointType(new DecimalType(latitude / 1000000.0 * (ns == 'N' ? 1 : -1)), new DecimalType(longitude / 1000000.0 * (ew == 'E' ? 1 : -1)))"}],
            'is_trigger_channel': False,
            'init_code': 'this.setCoordinatesCallbackPeriod(channelCfg.updateInterval);',
            'dispose_code': 'this.setCoordinatesCallbackPeriod(0);'
        }, {
            'id': 'Fix',
            'type': 'Fix',

            'getters': [{
                'packet': 'Get Status',
                'packet_params': [],
                'transform': "value.hasFix ? OnOffType.ON : OnOffType.OFF"}],

            'callbacks': [{
                'packet': 'Status',
                'transform': "hasFix ? OnOffType.ON : OnOffType.OFF"}],

            'is_trigger_channel': False,
            'init_code': 'this.setStatusCallbackPeriod(channelCfg.updateInterval);',
            'dispose_code': 'this.setStatusCallbackPeriod(0);'
        }, {
            'id': 'Satellites In View',
            'type': 'SatellitesInView',

            'getters': [{
                'packet': 'Get Status',
                'packet_params': [],
                'transform': "new QuantityType(value.satellitesView, SmartHomeUnits.ONE)"}],

            'callbacks': [{
                'packet': 'Status',
                'transform': "new QuantityType(satellitesView, SmartHomeUnits.ONE)"}],

            'is_trigger_channel': False
        },  {
            'id': 'Altitude',
            'type': 'Altitude',

            'getters': [{
                'packet': 'Get Altitude',
                'packet_params': [],
                'transform': "new QuantityType(value.altitude / 100.0, SIUnits.METRE)"}],

            'callbacks': [{
                'packet': 'Altitude',
                'transform': "new QuantityType(altitude / 100.0, SIUnits.METRE)"}],

            'is_trigger_channel': False,
            'init_code': 'this.setAltitudeCallbackPeriod(channelCfg.updateInterval);',
            'dispose_code': 'this.setAltitudeCallbackPeriod(0);'
        },  {
            'id': 'Geoidal Separation',
            'type': 'GeoidalSeparation',

            'getters': [{
                'packet': 'Get Altitude',
                'packet_params': [],
                'transform': "new QuantityType(value.geoidalSeparation / 100.0, SIUnits.METRE)"}],

            'callbacks': [{
                'packet': 'Altitude',
                'transform': "new QuantityType(geoidalSeparation / 100.0, SIUnits.METRE)"}],

            'is_trigger_channel': False
        },  {
            'id': 'Course',
            'type': 'Course',

            'getters': [{
                'packet': 'Get Motion',
                'packet_params': [],
                'transform': "new QuantityType(value.course / 100.0, SmartHomeUnits.DEGREE_ANGLE)"}],

            'callbacks': [{
                'packet': 'Motion',
                'transform': "new QuantityType(course / 100.0, SmartHomeUnits.DEGREE_ANGLE)"}],

            'is_trigger_channel': False,
            'init_code': 'this.setMotionCallbackPeriod(channelCfg.updateInterval);',
            'dispose_code': 'this.setMotionCallbackPeriod(0);'
        },  {
            'id': 'Speed',
            'type': 'Speed',

            'getters': [{
                'packet': 'Get Motion',
                'packet_params': [],
                'transform': "new QuantityType(value.speed / 100.0, SIUnits.KILOMETRE_PER_HOUR)"}],

            'callbacks': [{
                'packet': 'Motion',
                'transform': "new QuantityType(speed / 100.0, SIUnits.KILOMETRE_PER_HOUR)"}],

            'is_trigger_channel': False
        },  {
            'id': 'Date Time',
            'type': 'DateTime',

            'getters': [{
                'packet': 'Get Date Time',
                'packet_params': [],
                'transform': 'new DateTimeType(Helper.parseGPSDateTime(value.date, value.time).withZoneSameInstant(ZoneId.systemDefault()))'}],

            'callbacks': [{
                'packet': 'Date Time',
                'transform': "new DateTimeType(Helper.parseGPSDateTime(date, time).withZoneSameInstant(ZoneId.systemDefault()))"}],

            'is_trigger_channel': False,
            'init_code': 'this.setDateTimeCallbackPeriod(channelCfg.updateInterval);',
            'dispose_code': 'this.setDateTimeCallbackPeriod(0);'
        }, {
            'id': 'Pulse Per Second',
            'type': 'system.trigger',

            'label': 'Pulse per Second',
            'description': 'This channel is triggered precisely once per second, see <a href=https://en.wikipedia.org/wiki/Pulse-per-second_signal>PPS</a>.<br/><br/>The precision of two subsequent pulses will be skewed because of the latency in the USB/RS485/Ethernet connection. But in the long run this will be very precise. For example a count of 3600 pulses will take exactly 1 hour.',

            'callbacks': [{
                'packet': 'Pulse Per Second',
                'transform': 'CommonTriggerEvents.PRESSED'}],

            'is_trigger_channel': True
        }, {
            'id': 'Restart',
            'type': 'Restart',

            'setters': [{
                'packet': 'Restart',
                'packet_params': ['Integer.valueOf(cmd.toString())']}],
            'setter_command_type': "StringType"
        }
    ],
    'channel_types': [
        oh_generic_channel_type('Location', 'Location', 'Location',
                     description='The location as determined by the bricklet.',
                     read_only=True),
        oh_generic_channel_type('Fix', 'Switch', 'Fix',
                     description='If enabled, a fix is currently available',
                     read_only=True),
        oh_generic_channel_type('SatellitesInView', 'Number:Dimensionless', 'Satellites In View',
                     description='The number of satellites that are in view.',
                     read_only=True,
                     pattern='%d %unit%'),
        oh_generic_channel_type('Altitude', 'Number:Length', 'Altitude',
                     description='The current altitude',
                     read_only=True,
                     pattern='%.2f %unit%'),
        oh_generic_channel_type('GeoidalSeparation', 'Number:Length', 'Geoidal Separation',
                     description='The geoidal separation corresponding to the current altitude',
                     read_only=True,
                     pattern='%.2f %unit%'),
        oh_generic_channel_type('Course', 'Number:Angle', 'Course',
                     description='The current course. A course of 0° means the Bricklet is traveling north bound and 90° means it is traveling east bound. Please note that this only returns useful values if an actual movement is present.',
                     read_only=True,
                     pattern='%.2f %unit%'),
        oh_generic_channel_type('Speed', 'Number:Speed', 'Speed',
                     description='The current speed. Please note that this only returns useful values if an actual movement is present.',
                     read_only=True,
                     pattern='%.2f'),
        oh_generic_channel_type('DateTime', 'DateTime', 'Date Time',
                     description='The current date and time.',
                     read_only=True),
        oh_generic_channel_type('Restart', 'String', 'Restart',
                    description="Restarts the GPS Bricklet, the following restart types are available:<ul><li>Hot start (use all available data in the NV store)</li> <li>Warm start (don't use ephemeris at restart)</li> <li>Cold start (don't use time, position, almanacs and ephemeris at restart)</li> <li>Factory reset (clear all system/user configurations at restart)</li></ul>",
                    command_options=[('Hot Start', '0'),
                                     ('Warm Start', '1'),
                                     ('Cold Start', '2'),
                                     ('Factory reset', '3')])
    ]
}
