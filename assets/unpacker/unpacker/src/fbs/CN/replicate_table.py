# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class enum__Torappu_ItemType(object):
    NONE = 0
    CHAR = 1
    CARD_EXP = 2
    MATERIAL = 3
    GOLD = 4
    EXP_PLAYER = 5
    TKT_TRY = 6
    TKT_RECRUIT = 7
    TKT_INST_FIN = 8
    TKT_GACHA = 9
    ACTIVITY_COIN = 10
    DIAMOND = 11
    DIAMOND_SHD = 12
    HGG_SHD = 13
    LGG_SHD = 14
    FURN = 15
    AP_GAMEPLAY = 16
    AP_BASE = 17
    SOCIAL_PT = 18
    CHAR_SKIN = 19
    TKT_GACHA_10 = 20
    TKT_GACHA_PRSV = 21
    AP_ITEM = 22
    AP_SUPPLY = 23
    RENAMING_CARD = 24
    RENAMING_CARD_2 = 25
    ET_STAGE = 26
    ACTIVITY_ITEM = 27
    VOUCHER_PICK = 28
    VOUCHER_CGACHA = 29
    VOUCHER_MGACHA = 30
    CRS_SHOP_COIN = 31
    CRS_RUNE_COIN = 32
    LMTGS_COIN = 33
    EPGS_COIN = 34
    LIMITED_TKT_GACHA_10 = 35
    LIMITED_FREE_GACHA = 36
    REP_COIN = 37
    ROGUELIKE = 38
    LINKAGE_TKT_GACHA_10 = 39
    VOUCHER_ELITE_II_4 = 40
    VOUCHER_ELITE_II_5 = 41
    VOUCHER_ELITE_II_6 = 42
    VOUCHER_SKIN = 43
    RETRO_COIN = 44
    PLAYER_AVATAR = 45
    UNI_COLLECTION = 46
    VOUCHER_FULL_POTENTIAL = 47
    RL_COIN = 48
    RETURN_CREDIT = 49
    MEDAL = 50
    CHARM = 51
    HOME_BACKGROUND = 52
    EXTERMINATION_AGENT = 53
    OPTIONAL_VOUCHER_PICK = 54
    ACT_CART_COMPONENT = 55
    VOUCHER_LEVELMAX_6 = 56
    VOUCHER_LEVELMAX_5 = 57
    VOUCHER_LEVELMAX_4 = 58
    VOUCHER_SKILL_SPECIALLEVELMAX_6 = 59
    VOUCHER_SKILL_SPECIALLEVELMAX_5 = 60
    VOUCHER_SKILL_SPECIALLEVELMAX_4 = 61
    ACTIVITY_POTENTIAL = 62
    ITEM_PACK = 63
    SANDBOX = 64
    FAVOR_ADD_ITEM = 65
    CLASSIC_SHD = 66
    CLASSIC_TKT_GACHA = 67
    CLASSIC_TKT_GACHA_10 = 68
    LIMITED_BUFF = 69
    CLASSIC_FES_PICK_TIER_5 = 70
    CLASSIC_FES_PICK_TIER_6 = 71
    RETURN_PROGRESS = 72
    NEW_PROGRESS = 73
    MCARD_VOUCHER = 74
    MATERIAL_ISSUE_VOUCHER = 75
    CRS_SHOP_COIN_V2 = 76
    HOME_THEME = 77
    SANDBOX_PERM = 78
    SANDBOX_TOKEN = 79
    TEMPLATE_TRAP = 80
    NAME_CARD_SKIN = 81
    EMOTICON_SET = 82
    EXCLUSIVE_TKT_GACHA = 83
    EXCLUSIVE_TKT_GACHA_10 = 84


class clz_Torappu_ItemBundle(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_ItemBundle()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_ItemBundle(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_ItemBundle
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_ItemBundle
    def Id(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ItemBundle
    def Count(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_ItemBundle
    def Type(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

def clz_Torappu_ItemBundleStart(builder):
    builder.StartObject(3)

def clz_Torappu_ItemBundleAddId(builder, id):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(id), 0)

def clz_Torappu_ItemBundleAddCount(builder, count):
    builder.PrependInt32Slot(1, count, 0)

def clz_Torappu_ItemBundleAddType(builder, type):
    builder.PrependInt32Slot(2, type, 0)

def clz_Torappu_ItemBundleEnd(builder):
    return builder.EndObject()



class clz_Torappu_ReplicateData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_ReplicateData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_ReplicateData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_ReplicateData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_ReplicateData
    def Item(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            obj = clz_Torappu_ItemBundle()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_ReplicateData
    def ReplicateTokenItem(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            obj = clz_Torappu_ItemBundle()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

def clz_Torappu_ReplicateDataStart(builder):
    builder.StartObject(2)

def clz_Torappu_ReplicateDataAddItem(builder, item):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(item), 0)

def clz_Torappu_ReplicateDataAddReplicateTokenItem(builder, replicateTokenItem):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(replicateTokenItem), 0)

def clz_Torappu_ReplicateDataEnd(builder):
    return builder.EndObject()



class clz_Torappu_ReplicateTable(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_ReplicateTable()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_ReplicateTable(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_ReplicateTable
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_ReplicateTable
    def ReplicateList(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = clz_Torappu_ReplicateData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_ReplicateTable
    def ReplicateListLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_ReplicateTable
    def ReplicateListIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

def clz_Torappu_ReplicateTableStart(builder):
    builder.StartObject(1)

def clz_Torappu_ReplicateTableAddReplicateList(builder, replicateList):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(replicateList), 0)

def clz_Torappu_ReplicateTableStartReplicateListVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_ReplicateTableEnd(builder):
    return builder.EndObject()



class dict__string__clz_Torappu_ReplicateTable(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = dict__string__clz_Torappu_ReplicateTable()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsdict__string__clz_Torappu_ReplicateTable(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # dict__string__clz_Torappu_ReplicateTable
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # dict__string__clz_Torappu_ReplicateTable
    def Key(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # dict__string__clz_Torappu_ReplicateTable
    def Value(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            obj = clz_Torappu_ReplicateTable()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

def dict__string__clz_Torappu_ReplicateTableStart(builder):
    builder.StartObject(2)

def dict__string__clz_Torappu_ReplicateTableAddKey(builder, key):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(key), 0)

def dict__string__clz_Torappu_ReplicateTableAddValue(builder, value):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(value), 0)

def dict__string__clz_Torappu_ReplicateTableEnd(builder):
    return builder.EndObject()



class clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable
    def Replications(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = dict__string__clz_Torappu_ReplicateTable()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable
    def ReplicationsLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable
    def ReplicationsIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

def clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTableStart(builder):
    builder.StartObject(1)

def clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTableAddReplications(builder, replications):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(replications), 0)

def clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTableStartReplicationsVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTableEnd(builder):
    return builder.EndObject()

ROOT_TYPE = clz_Torappu_SimpleKVTable_clz_Torappu_ReplicateTable
