# Overview

## Instruments and control flow

```text
╭ ─ ─ ─ ─    ╭ ─ ─ ─ ─ ─ ─ ╮   ╭ ─ ─ ─ ─ ╮   ╭ ─ ─ ─ ─ ╮   ╭ ─ ─ ─ ─    ╭ ─ ─ ─ ╮
  Baking │    Cooler safety     Flowmeter       Light        Valves │      VCT   
╰ ─ ─ ─ ─    ╰ ─ ─ ─ ─ ─ ─ ╯   ╰ ─ ─ ─ ─ ╯   ╰ ─ ─ ─ ─ ╯   ╰ ─ ─ ─ ─    ╰ ─ ─ ─ ╯
     △              │               │             △            △            △    
     │              │               └──┐  ┌───────┘            │            │    
     │              └────────────────┐ │  │ ┌──────────────────┘            │    
     └─────────────────────────────┐ │ │  │ │ ┌─────────────────────────────┘    
                                   │ ▽ ▽  │ ▽ ▽                                  
                                 ╭──────────────╮                                
                                 │Control board │                                
                                 ╰──────────────╯                                
                                         ▲                                       
                                         ┃ Poststation (USB)                     
                                         ▼                                       
                                   ╔══════════╗                                  
                     USB           ║   Host   ║          LAN                     
          ┌───────────────────────▷║----------║◁─────────────────────┐           
          │                        ║reTerminal║                      ▽     Moxa  
          │                        ╚══════════╝          ╭─────────────────────╮ 
          │                                              │░░░░░░░░░░░░░░░░░░░░░│ 
          │                                              │░░░┌ ─ ─ ─ ─ ─ ─ ┐░░░│ 
          │                                              │░░░   Ion pump    ░░░│ 
          │                                              │░░░└ ─ ─ ─ ─ ─ ─ ┘░░░│ 
          │                                              │░░░┌ ─ ─ ─ ─ ─ ─ ┐░░░│ 
          │                                              │░░░  Cryocooler   ░░░│ 
          │                                              │░░░└ ─ ─ ─ ─ ─ ─ ┘░░░│ 
          ▽                                              │░░░┌ ─ ─ ─ ─ ─ ─ ┐░░░│ 
 ╭ ─ ─ ─ ─ ─ ─ ─ ─ ╮                                     │░░░     Gauge     ░░░│ 
     Temperature                   ╭ ─ ─ ─ ─ ─ ╮   LAN   │░░░│ controller  │░░░│ 
 │   controller    │                 Pumpstand  ◁───────▷│░░░ ─ ─ ─ ─ ─ ─ ─ ░░░│ 
   (instrumentRs)                  │  (OPCUA)  │         │░░░░░░░░░░░░░░░░░░░░░│ 
 ╰ ─ ─ ─ ─ ─ ─ ─ ─ ╯                ─ ─ ─ ─ ─ ─          ╰─────────────────────╯ 
                                                                 (instrumentRs)
```

Above diagram shows an overview of the connected devices and the control flow.
The host computer, a reTerminal 10" touch screen device
based on a RaspberryPi compute module sits at the center of control.
It runs the host software, a touch interface (based on [Slint](https://slint.dev/))
that the user can use to interact with the cryostorage chamber.

The control board, our in-house designed PCB to control various devices,
contains a Raspberry Pi Pico2 that serves as the MCU.
This MCU runs the firmware that communicates (1) with the connected devices
and (2) via [Poststation](https://poststation.rs)
with the host computer.

Furthermore, the host computer is also connected via USB to a Lakeshore 336 
temperature controller. 

The rest of the instruments, i.e., the ion pump controller, cryocooler, 
gauge controller, and pumpstand are connected via LAN to the host.
While the pumpstand is directly connected via an ethernet cable, 
the other three instruments are connected to a MOXA NPort 5650-8-DT.
Details on the LAN and MOXA configuration can be found 
in [hardware setup](./setup.md).

While the pumpstand serves an OPCUA server that we communicate with directly,
all other directly connected instruments that do not hang on the control board
use in-house drivers that are based on 
[`instrumentRs`](https://docs.rs/instrumentrs/latest/instrumentrs/).

## Terminology

- Controller: The control board designed by us that runs via a Raspberry Pi Pico2.
- Firmware: Refers to the software that runs on the controller board (Pico2).
- Software: Refers to the `host` software that runs on the Raspberry Pi ReTerminal.
