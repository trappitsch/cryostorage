# VCT

## Handshake

The VCT control box has a DB-15 plug that usually connects to an SEM.
In our case, this DB-15 plug is connected to the control board.

> [!NOTE]
> The setup below is equal to what Zeiss SEMs use to connect to the VCT.
> Thus, this handshake is also known as the Zeiss handshake.

```text
                                  VCT DB15    │         Control board             
                                 ╭─────────╮                                      
                                 │░░░░░░░░░│  │                                   
                                 │░░┌───┐░░│              Gate open               
                         ┌───────┼──┤ 1 │░░│  │         ┌────────────┐         R=∞
           Output      \         │░░└───┘░░│            │            │            
      Gate closed       \        │░░░░░░░░░│  │         │            │            
                         \       │░░┌───┐░░│            │            │            
                         └───────┼──┤ 2 │░░│  │   ──────┘            └──────   R=0
                                 │░░└───┘░░│                                      
                                 │░░░░░░░░░│  │                                   
                                 │░░░░░░░░░│                                      
                                 │░░┌───┐░░│  │          VCT attached             
                         ┌───────┼──┤ 4 │░░│           ┌──────────────┐        R=∞
           Output      \         │░░└───┘░░│  │        │              │           
 Attach procedure       \        │░░░░░░░░░│           │              │           
                         \       │░░┌───┐░░│  │        │              │           
                         └───────┼──┤ 5 │░░│      ─────┘              └─────   R=0
                                 │░░└───┘░░│  │                                   
                                 │░░░░░░░░░│                                      
                                 │░░░░░░░░░│  │                                   
                        Vcc      │░░┌───┐░░│                                      
                         ├───────┼──┤ 9 │░░│  │   ────┐                ┌────   R=∞
            Input                │░░└───┘░░│          │                │          
    Chamber ready                │░░░░░░░░░│  │       │                │          
                                 │░░┌───┐░░│          │                │          
                         ├───────┼──┤ 15│░░│  │       └────────────────┘       R=0
                        GND      │░░└───┘░░│                ready                 
                                 │░░░░░░░░░│  │                                   
                                 ╰─────────╯                                      
                                              │
```

Above figure shows a schematic of the DB-15 plug on the left
and the respective signals to/from the control board on the right.
From the point of view of the control board,
the connections can be described as following:

| Pins connected | Control board |              Function             |
|:--------------:|:-------------:|:---------------------------------:|
|     1 and 2    |     Input     | Check if the gate is open.        |
|     4 and 5    |     Input     | Check if the VCT is attached.     |
|    9 and 15    |     Output    | Signal that the chamber is ready. |

### Gate open

Pins 1 and 2 represent the status if the VCT's gate vale is open or closed.
If the gate valve is closed, these two pins are shorted.
When the VCT however opens the gate vale, the switch between these two pins
is opened and thus we measure an infinite resistance between them
on the controller.

### Attach procedure

Pins 4 and 5 represent the status of the attach procedure.
If the VCT is detached, these two pins are shorted.
Attaching the VCT however using the Leicha touchscreen
opens the switch between these two pins and thus we read
an infinite resistance on the control board.

### Chamber ready

Pins 9 and 15 allow us to signal to the VCT, these pins are thus controlled
by the control board.
To signal readyness, the control board shorts these to pins together.
If the switch between these two pins is opened,
the VCT assumes the chamber is not ready for a transfer.
It will then not allow the user to attach the shuttle to the dock.

## Sample transfer procedure

In terms of a typical sample transfer,
the following takes place in terms of the steps above:

1. The chamber signals to the VCT that it is ready to transfer a sample
   by shorting pins 9 and 15.
2. The VCT then allows the user to attach the shuttle by pressing
   the respective button on the touch screen.
3. When the attach procedure starts, the resistance between pins 4 and 5
   jumps to infinity, signaling to the control board that the attach procedure
   has started.
4. Once ready, the VCT will open its gate valve.
   When this happens, the resistance between pins 1 and 2 will jump to infinity,
   signaling to the control board that the gate valve is open.

> [!IMPORTANT]
> This gate valve is open signal is used as the signal to not allow our transfer
> valve to close. More information on this safety and how it is implemented
> can be found in the [System Safety](./safety.md) section.
