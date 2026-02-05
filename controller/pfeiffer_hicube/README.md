# Pfeiffer HiCube driver

## OPC UA Information

### Some commands:

The following values were generated with the OPC UA client when toggling various things.

**Stop turbo pump**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P1_010_PumpgStatn 
with value: 
    DataValue(
        Value=Variant(
            Value=False, 
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None

)')
```

**Start turbo pump**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P1_010_PumpgStatn 
with value: 
    DataValue(
        Value=Variant(
            Value=True, 
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None

)')
```

**Enable Venting valve**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P1_012_EnableVent 
with value: 
    DataValue(
        Value=Variant(
            Value=True, 
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None
    )
')```

**Disable Venting valve**

```uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P1_012_EnableVent 
with value: 
    DataValue(
        Value=Variant(
            Value=False,
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False
        ),
        StatusCode_=StatusCode(value=0), 
    SourceTimestamp=None,
    ServerTimestamp=None,
    SourcePicoseconds=None,
ServerPicoseconds=None
    )
')```

**Stop roughing pump**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P2_010_PumpgStatn 
with value: 
    DataValue(
        Value=Variant(
            Value=False, 
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None
    )
')
```

**Start roughing pump**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=P2_010_PumpgStatn 
with value: 
    DataValue(
        Value=Variant(
            Value=True, 
            VariantType=<VariantType.Boolean: 1>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None

)')
```

**Set pump stand status to 4.0 (off)**

This command is just not accepted.

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=SYS_STATUS 
with value: 
    DataValue(
        Value=Variant(
            Value=4.0, 
            VariantType=<VariantType.Float: 10>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None

)')
```
**Set pump stand status to 1.0 (on)**

```
uawidgets.attrs_widget - INFO - 
Writing attribute 13 of node 
    ns=1;s=SYS_STATUS 
with value: 
    DataValue(
        Value=Variant(
            Value=1.0, 
            VariantType=<VariantType.Float: 10>, 
            Dimensions=None, 
            is_array=False
        ), 
        StatusCode_=StatusCode(value=0), 
        SourceTimestamp=None, 
        ServerTimestamp=None, 
        SourcePicoseconds=None, 
        ServerPicoseconds=None

)')
```
