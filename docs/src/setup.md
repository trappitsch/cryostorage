# Hardware Setup

This section summarizes briefly the hardware setup,
i.e., how instruments are connected to the host computer
and how they are configured.

## Host computer - ReTerminal

Ethernet configuration

- Static IP: 192.168.1.1
- Subnet mask: 255.255.255.0

Scripts that are used on the ReTerminal to hide the top menu bar,
aid in properly displaying the GUI can be found in the repo's `reterminal` folder.

## Moxa

The Moxa we are using is a NPort 5650-8-DT.

IP setup:

- Static IP: 192.168.1.2
- Subnet mask: 255.255.255.0

The following ports are configured:

- Port 1:
  - Agilent 4UHV Ion pump controller
  - Serial setup: 38_400, 8, N, 1
  - Mode: TCP Server
  - IP Port: 4001

- Port 2:
  - Pfeiffer Omnicontrol 200 gauge controller
  - RS-485, 2wire setup: 9_600, 8, N, 1
  - Moxa pull-up resistors on both lines set to 1 kOhm (see Appendix B of Moxa Manual)
  - Mode: TCP Server
  - IP Port: 4002

- Port 3:
  - Sunpower CryoTel GT Cryocooler
  - Serial setup: 4_800, 8, N, 1
  - Mode: TCP Server
  - IP Port: 4003

## Pfeiffer HiCube Pump stand

The HiCube is connected to the second LAN port of the Moxa.
The host computer is connected to the other LAN port of the Moxa.
With the static IP setup as we currently have it,
the HiCube can be accessed from the host computer, i.e.,
the Moxa acts as a switch.

- Static IP: 192.168.1.100

> [!NOTE]
> The firmware of the HiCube can only be updated by connecting the HiCube to the
> internet. This has to be done actively if necessary, as no internet access is
> provided via the ReTerminal.

## Lakeshore 336

The Lakeshore 336 is connected to the ReTerminal via a USB port.

## Control board

Our own control board is connected via USB to the ReTerminal.
The host software talks to it via [poststation](https://poststation.rs/).
The poststation TUI can also be used to observe broadcasts and test commands manually.
