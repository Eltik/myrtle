# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class enum__Torappu_StoryReviewEntryType(object):
    NONE = 0
    ACTIVITY = 1
    MINI_ACTIVITY = 2
    MAINLINE = 3


class enum__Torappu_StoryReviewType(object):
    NONE = 0
    ACTIVITY_STORY = 1
    MINI_STORY = 2
    MAIN_STORY = 3


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


class enum__Torappu_StoryReviewUnlockType(object):
    STAGE_CLEAR = 0
    USE_ITEM = 1
    BY_START_TIME = 2
    NOTHING = 3


class enum__Torappu_PlayerStageState(object):
    UNLOCKED = 0
    PLAYED = 1
    PASS = 2
    COMPLETE = 3


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



class clz_Torappu_StoryData_Condition_StageCondition(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_StoryData_Condition_StageCondition()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_StoryData_Condition_StageCondition(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_StoryData_Condition_StageCondition
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_StoryData_Condition_StageCondition
    def StageId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryData_Condition_StageCondition
    def MinState(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryData_Condition_StageCondition
    def MaxState(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

def clz_Torappu_StoryData_Condition_StageConditionStart(builder):
    builder.StartObject(3)

def clz_Torappu_StoryData_Condition_StageConditionAddStageId(builder, stageId):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(stageId), 0)

def clz_Torappu_StoryData_Condition_StageConditionAddMinState(builder, minState):
    builder.PrependInt32Slot(1, minState, 0)

def clz_Torappu_StoryData_Condition_StageConditionAddMaxState(builder, maxState):
    builder.PrependInt32Slot(2, maxState, 0)

def clz_Torappu_StoryData_Condition_StageConditionEnd(builder):
    return builder.EndObject()



class clz_Torappu_StoryReviewInfoClientData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_StoryReviewInfoClientData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_StoryReviewInfoClientData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_StoryReviewInfoClientData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_StoryReviewInfoClientData
    def StoryReviewType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def StoryId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryGroup(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StorySort(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(10))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def StoryDependence(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(12))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryCanShow(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(14))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def StoryCode(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(16))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryName(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(18))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryPic(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(20))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryInfo(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(22))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def StoryCanEnter(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(24))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def StoryTxt(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(26))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def AvgTag(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(28))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def UnLockType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(30))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def CostItemType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(32))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def CostItemId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(34))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def CostItemCount(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(36))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def StageCount(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(38))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def RequiredStages(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(40))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = clz_Torappu_StoryData_Condition_StageCondition()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_StoryReviewInfoClientData
    def RequiredStagesLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(40))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_StoryReviewInfoClientData
    def RequiredStagesIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(40))
        return o == 0

def clz_Torappu_StoryReviewInfoClientDataStart(builder):
    builder.StartObject(19)

def clz_Torappu_StoryReviewInfoClientDataAddStoryReviewType(builder, storyReviewType):
    builder.PrependInt32Slot(0, storyReviewType, 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryId(builder, storyId):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(storyId), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryGroup(builder, storyGroup):
    builder.PrependUOffsetTRelativeSlot(2, flatbuffers.number_types.UOffsetTFlags.py_type(storyGroup), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStorySort(builder, storySort):
    builder.PrependInt32Slot(3, storySort, 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryDependence(builder, storyDependence):
    builder.PrependUOffsetTRelativeSlot(4, flatbuffers.number_types.UOffsetTFlags.py_type(storyDependence), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryCanShow(builder, storyCanShow):
    builder.PrependInt32Slot(5, storyCanShow, 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryCode(builder, storyCode):
    builder.PrependUOffsetTRelativeSlot(6, flatbuffers.number_types.UOffsetTFlags.py_type(storyCode), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryName(builder, storyName):
    builder.PrependUOffsetTRelativeSlot(7, flatbuffers.number_types.UOffsetTFlags.py_type(storyName), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryPic(builder, storyPic):
    builder.PrependUOffsetTRelativeSlot(8, flatbuffers.number_types.UOffsetTFlags.py_type(storyPic), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryInfo(builder, storyInfo):
    builder.PrependUOffsetTRelativeSlot(9, flatbuffers.number_types.UOffsetTFlags.py_type(storyInfo), 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryCanEnter(builder, storyCanEnter):
    builder.PrependInt32Slot(10, storyCanEnter, 0)

def clz_Torappu_StoryReviewInfoClientDataAddStoryTxt(builder, storyTxt):
    builder.PrependUOffsetTRelativeSlot(11, flatbuffers.number_types.UOffsetTFlags.py_type(storyTxt), 0)

def clz_Torappu_StoryReviewInfoClientDataAddAvgTag(builder, avgTag):
    builder.PrependUOffsetTRelativeSlot(12, flatbuffers.number_types.UOffsetTFlags.py_type(avgTag), 0)

def clz_Torappu_StoryReviewInfoClientDataAddUnLockType(builder, unLockType):
    builder.PrependInt32Slot(13, unLockType, 0)

def clz_Torappu_StoryReviewInfoClientDataAddCostItemType(builder, costItemType):
    builder.PrependInt32Slot(14, costItemType, 0)

def clz_Torappu_StoryReviewInfoClientDataAddCostItemId(builder, costItemId):
    builder.PrependUOffsetTRelativeSlot(15, flatbuffers.number_types.UOffsetTFlags.py_type(costItemId), 0)

def clz_Torappu_StoryReviewInfoClientDataAddCostItemCount(builder, costItemCount):
    builder.PrependInt32Slot(16, costItemCount, 0)

def clz_Torappu_StoryReviewInfoClientDataAddStageCount(builder, stageCount):
    builder.PrependInt32Slot(17, stageCount, 0)

def clz_Torappu_StoryReviewInfoClientDataAddRequiredStages(builder, requiredStages):
    builder.PrependUOffsetTRelativeSlot(18, flatbuffers.number_types.UOffsetTFlags.py_type(requiredStages), 0)

def clz_Torappu_StoryReviewInfoClientDataStartRequiredStagesVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_StoryReviewInfoClientDataEnd(builder):
    return builder.EndObject()



class clz_Torappu_StoryReviewGroupClientData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_StoryReviewGroupClientData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_StoryReviewGroupClientData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_StoryReviewGroupClientData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_StoryReviewGroupClientData
    def Id(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def Name(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def EntryType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def ActType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(10))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def StartTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(12))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def EndTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(14))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def StartShowTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(16))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def EndShowTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(18))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def RemakeStartTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(20))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def RemakeEndTime(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(22))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int64Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def StoryEntryPicId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(24))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def StoryPicId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(26))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def StoryMainColor(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(28))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def CustomType(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(30))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def StoryCompleteMedalId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(32))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def Rewards(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(34))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = clz_Torappu_ItemBundle()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def RewardsLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(34))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def RewardsIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(34))
        return o == 0

    # clz_Torappu_StoryReviewGroupClientData
    def InfoUnlockDatas(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(36))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = clz_Torappu_StoryReviewInfoClientData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_StoryReviewGroupClientData
    def InfoUnlockDatasLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(36))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_StoryReviewGroupClientData
    def InfoUnlockDatasIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(36))
        return o == 0

def clz_Torappu_StoryReviewGroupClientDataStart(builder):
    builder.StartObject(17)

def clz_Torappu_StoryReviewGroupClientDataAddId(builder, id):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(id), 0)

def clz_Torappu_StoryReviewGroupClientDataAddName(builder, name):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(name), 0)

def clz_Torappu_StoryReviewGroupClientDataAddEntryType(builder, entryType):
    builder.PrependInt32Slot(2, entryType, 0)

def clz_Torappu_StoryReviewGroupClientDataAddActType(builder, actType):
    builder.PrependInt32Slot(3, actType, 0)

def clz_Torappu_StoryReviewGroupClientDataAddStartTime(builder, startTime):
    builder.PrependInt64Slot(4, startTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddEndTime(builder, endTime):
    builder.PrependInt64Slot(5, endTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddStartShowTime(builder, startShowTime):
    builder.PrependInt64Slot(6, startShowTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddEndShowTime(builder, endShowTime):
    builder.PrependInt64Slot(7, endShowTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddRemakeStartTime(builder, remakeStartTime):
    builder.PrependInt64Slot(8, remakeStartTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddRemakeEndTime(builder, remakeEndTime):
    builder.PrependInt64Slot(9, remakeEndTime, 0)

def clz_Torappu_StoryReviewGroupClientDataAddStoryEntryPicId(builder, storyEntryPicId):
    builder.PrependUOffsetTRelativeSlot(10, flatbuffers.number_types.UOffsetTFlags.py_type(storyEntryPicId), 0)

def clz_Torappu_StoryReviewGroupClientDataAddStoryPicId(builder, storyPicId):
    builder.PrependUOffsetTRelativeSlot(11, flatbuffers.number_types.UOffsetTFlags.py_type(storyPicId), 0)

def clz_Torappu_StoryReviewGroupClientDataAddStoryMainColor(builder, storyMainColor):
    builder.PrependUOffsetTRelativeSlot(12, flatbuffers.number_types.UOffsetTFlags.py_type(storyMainColor), 0)

def clz_Torappu_StoryReviewGroupClientDataAddCustomType(builder, customType):
    builder.PrependInt32Slot(13, customType, 0)

def clz_Torappu_StoryReviewGroupClientDataAddStoryCompleteMedalId(builder, storyCompleteMedalId):
    builder.PrependUOffsetTRelativeSlot(14, flatbuffers.number_types.UOffsetTFlags.py_type(storyCompleteMedalId), 0)

def clz_Torappu_StoryReviewGroupClientDataAddRewards(builder, rewards):
    builder.PrependUOffsetTRelativeSlot(15, flatbuffers.number_types.UOffsetTFlags.py_type(rewards), 0)

def clz_Torappu_StoryReviewGroupClientDataStartRewardsVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_StoryReviewGroupClientDataAddInfoUnlockDatas(builder, infoUnlockDatas):
    builder.PrependUOffsetTRelativeSlot(16, flatbuffers.number_types.UOffsetTFlags.py_type(infoUnlockDatas), 0)

def clz_Torappu_StoryReviewGroupClientDataStartInfoUnlockDatasVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_StoryReviewGroupClientDataEnd(builder):
    return builder.EndObject()



class dict__string__clz_Torappu_StoryReviewGroupClientData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = dict__string__clz_Torappu_StoryReviewGroupClientData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsdict__string__clz_Torappu_StoryReviewGroupClientData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # dict__string__clz_Torappu_StoryReviewGroupClientData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # dict__string__clz_Torappu_StoryReviewGroupClientData
    def Key(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # dict__string__clz_Torappu_StoryReviewGroupClientData
    def Value(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            obj = clz_Torappu_StoryReviewGroupClientData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

def dict__string__clz_Torappu_StoryReviewGroupClientDataStart(builder):
    builder.StartObject(2)

def dict__string__clz_Torappu_StoryReviewGroupClientDataAddKey(builder, key):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(key), 0)

def dict__string__clz_Torappu_StoryReviewGroupClientDataAddValue(builder, value):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(value), 0)

def dict__string__clz_Torappu_StoryReviewGroupClientDataEnd(builder):
    return builder.EndObject()



class clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData
    def StoryReviews(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = dict__string__clz_Torappu_StoryReviewGroupClientData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData
    def StoryReviewsLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData
    def StoryReviewsIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

def clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientDataStart(builder):
    builder.StartObject(1)

def clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientDataAddStoryReviews(builder, storyReviews):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(storyReviews), 0)

def clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientDataStartStoryReviewsVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientDataEnd(builder):
    return builder.EndObject()

ROOT_TYPE = clz_Torappu_SimpleKVTable_clz_Torappu_StoryReviewGroupClientData
