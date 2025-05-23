# automatically generated by the FlatBuffers compiler, do not modify

# namespace: 

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class clz_Torappu_ChapterData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_ChapterData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_ChapterData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_ChapterData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_ChapterData
    def ChapterId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def ChapterName(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def ChapterName2(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def ChapterIndex(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(10))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Int32Flags, o + self._tab.Pos)
        return 0

    # clz_Torappu_ChapterData
    def PreposedChapterId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(12))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def StartZoneId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(14))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def EndZoneId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(16))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # clz_Torappu_ChapterData
    def ChapterEndStageId(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(18))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

def clz_Torappu_ChapterDataStart(builder):
    builder.StartObject(8)

def clz_Torappu_ChapterDataAddChapterId(builder, chapterId):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(chapterId), 0)

def clz_Torappu_ChapterDataAddChapterName(builder, chapterName):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(chapterName), 0)

def clz_Torappu_ChapterDataAddChapterName2(builder, chapterName2):
    builder.PrependUOffsetTRelativeSlot(2, flatbuffers.number_types.UOffsetTFlags.py_type(chapterName2), 0)

def clz_Torappu_ChapterDataAddChapterIndex(builder, chapterIndex):
    builder.PrependInt32Slot(3, chapterIndex, 0)

def clz_Torappu_ChapterDataAddPreposedChapterId(builder, preposedChapterId):
    builder.PrependUOffsetTRelativeSlot(4, flatbuffers.number_types.UOffsetTFlags.py_type(preposedChapterId), 0)

def clz_Torappu_ChapterDataAddStartZoneId(builder, startZoneId):
    builder.PrependUOffsetTRelativeSlot(5, flatbuffers.number_types.UOffsetTFlags.py_type(startZoneId), 0)

def clz_Torappu_ChapterDataAddEndZoneId(builder, endZoneId):
    builder.PrependUOffsetTRelativeSlot(6, flatbuffers.number_types.UOffsetTFlags.py_type(endZoneId), 0)

def clz_Torappu_ChapterDataAddChapterEndStageId(builder, chapterEndStageId):
    builder.PrependUOffsetTRelativeSlot(7, flatbuffers.number_types.UOffsetTFlags.py_type(chapterEndStageId), 0)

def clz_Torappu_ChapterDataEnd(builder):
    return builder.EndObject()



class dict__string__clz_Torappu_ChapterData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = dict__string__clz_Torappu_ChapterData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsdict__string__clz_Torappu_ChapterData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # dict__string__clz_Torappu_ChapterData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # dict__string__clz_Torappu_ChapterData
    def Key(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.String(o + self._tab.Pos)
        return None

    # dict__string__clz_Torappu_ChapterData
    def Value(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            obj = clz_Torappu_ChapterData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

def dict__string__clz_Torappu_ChapterDataStart(builder):
    builder.StartObject(2)

def dict__string__clz_Torappu_ChapterDataAddKey(builder, key):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(key), 0)

def dict__string__clz_Torappu_ChapterDataAddValue(builder, value):
    builder.PrependUOffsetTRelativeSlot(1, flatbuffers.number_types.UOffsetTFlags.py_type(value), 0)

def dict__string__clz_Torappu_ChapterDataEnd(builder):
    return builder.EndObject()



class clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsclz_Torappu_SimpleKVTable_clz_Torappu_ChapterData(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData
    def Chapters(self, j):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = self._tab.Vector(o)
            x += flatbuffers.number_types.UOffsetTFlags.py_type(j) * 4
            x = self._tab.Indirect(x)
            obj = dict__string__clz_Torappu_ChapterData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData
    def ChaptersLength(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            return self._tab.VectorLen(o)
        return 0

    # clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData
    def ChaptersIsNone(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        return o == 0

def clz_Torappu_SimpleKVTable_clz_Torappu_ChapterDataStart(builder):
    builder.StartObject(1)

def clz_Torappu_SimpleKVTable_clz_Torappu_ChapterDataAddChapters(builder, chapters):
    builder.PrependUOffsetTRelativeSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(chapters), 0)

def clz_Torappu_SimpleKVTable_clz_Torappu_ChapterDataStartChaptersVector(builder, numElems):
    return builder.StartVector(4, numElems, 4)

def clz_Torappu_SimpleKVTable_clz_Torappu_ChapterDataEnd(builder):
    return builder.EndObject()

ROOT_TYPE = clz_Torappu_SimpleKVTable_clz_Torappu_ChapterData
