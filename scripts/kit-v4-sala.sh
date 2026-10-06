#!/usr/bin/env bash
# kit-v4-sala.sh — las salas del kit v4 del modo presencial (sprint 005, ADR 020 §4).
#
# Lee el guion `docs/kit-de-prueba/presencial.json` y, por cada sala, escribe:
#   docs/kit-de-prueba/audio/<id>.wav           16 kHz, mono, 16 bits: la sala en UNA pista, como la oye el Mac
#   docs/kit-de-prueba/audio/<id>.tiempos.json  dónde empieza y acaba cada turno dentro del wav (la verdad del kit)
#
# Audio 100 % sintético: dos voces del sistema (`say`), una por papel, con la pausa del guion antes de cada turno y
# la ganancia de cada voz (la del cliente, más lejos del micrófono, más baja). Cero grabaciones de personas.
#
# Qué NO hace: no suena (`say -o` escribe un archivo), no abre el micrófono ni pide ningún permiso del Mac. Solo
# escribe en `docs/kit-de-prueba/audio/` y en una carpeta temporal que borra al salir.
#
# Los audios se VERSIONAN, no se regeneran en cada corrida: macOS cambia su catálogo de voces entre versiones y un
# kit que se regenerara solo mediría una voz distinta cada vez (`docs/kit-de-prueba/audio/LEEME.md`).
#
# Uso: scripts/kit-v4-sala.sh
set -euo pipefail
cd "$(dirname "$0")/.."
command -v say >/dev/null && command -v afconvert >/dev/null || { echo "kit-v4-sala: hace falta macOS (say y afconvert)" >&2; exit 1; }
TMP=$(mktemp -d "${TMPDIR:-/tmp}/kit-v4-sala.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

python3 - docs/kit-de-prueba/presencial.json docs/kit-de-prueba/audio "$TMP" "$(sw_vers -productVersion)" <<'PY'
import array, json, subprocess, sys, wave

guion, salida, tmp, macos = sys.argv[1:5]
kit = json.load(open(guion, encoding="utf-8"))
HZ = 16_000
UMBRAL = 328   # 1 % del fondo de escala: lo que queda por debajo al principio y al final de un clip es su silencio
MARGEN = HZ // 50  # 20 ms de margen al recortar
COLA_MS = 1_500

def leer(ruta):
    with wave.open(ruta, "rb") as w:
        assert w.getframerate() == HZ and w.getnchannels() == 1 and w.getsampwidth() == 2, ruta
        a = array.array("h")
        a.frombytes(w.readframes(w.getnframes()))
        return a

def recortar(a):
    """Quita el silencio que `say` deja al principio y al final: así la pausa del guion es la pausa real."""
    i = next((k for k, x in enumerate(a) if abs(x) > UMBRAL), 0)
    j = next((k for k in range(len(a) - 1, -1, -1) if abs(a[k]) > UMBRAL), len(a) - 1)
    return a[max(0, i - MARGEN): min(len(a), j + MARGEN)]

class Ruido:
    """El ruido de fondo de una sala callada, muy por debajo del suelo del VAD, y siempre el mismo (semilla fija)."""
    def __init__(self):
        self.x = 2_005
    def muestra(self):
        self.x = (1_103_515_245 * self.x + 12_345) % 2**31
        return (self.x % 33) - 16

for sala in kit["salas"]:
    ruido = Ruido()
    sale = array.array("h")
    def callar(ms):
        sale.extend(ruido.muestra() for _ in range(ms * HZ // 1000))
    tiempos = []
    for i, t in enumerate(sala["turnos"]):
        voz = sala["voces"][t["quien"]]
        aiff, wav = f"{tmp}/{sala['id']}-{i}.aiff", f"{tmp}/{sala['id']}-{i}.wav"
        subprocess.run(["say", "-v", voz, "-o", aiff, t["dice"]], check=True)
        subprocess.run(["afconvert", "-f", "WAVE", "-d", "LEI16@16000", "-c", "1", aiff, wav], check=True)
        clip = recortar(leer(wav))
        g = float(sala["ganancia"][t["quien"]])
        callar(int(t["pausaMs"]))
        desde = len(sale)
        sale.extend(max(-32768, min(32767, int(x * g))) for x in clip)
        tiempos.append({"turno": i, "quien": t["quien"], "desdeMs": desde * 1000 // HZ, "hastaMs": len(sale) * 1000 // HZ})
    callar(COLA_MS)
    with wave.open(f"{salida}/{sala['archivo']}", "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(HZ)
        w.writeframes(sale.tobytes())
    with open(f"{salida}/{sala['id']}.tiempos.json", "w", encoding="utf-8") as f:
        json.dump({
            "_leeme": "Generado por scripts/kit-v4-sala.sh desde docs/kit-de-prueba/presencial.json: dónde está cada turno dentro del wav. Es la verdad del kit (quién habló y cuándo); la app no la ve.",
            "sala": sala["id"], "archivo": sala["archivo"], "macos": macos, "voces": sala["voces"],
            "ganancia": sala["ganancia"], "duracionMs": len(sale) * 1000 // HZ, "turnos": tiempos,
        }, f, ensure_ascii=False, indent=2)
        f.write("\n")
    print(f"kit-v4-sala: {sala['archivo']} · {len(sala['turnos'])} turnos · {len(sale) / HZ:.1f} s")
PY
