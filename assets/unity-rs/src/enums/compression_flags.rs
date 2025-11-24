use bitflags::bitflags;

bitflags! {
    /// Compression type flags for Unity bundle files
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CompressionFlags: u32 {
        /// No compression
        const NONE = 0;
        /// LZMA compression
        const LZMA = 1;
        /// LZ4 compression
        const LZ4 = 2;
        /// LZ4HC compression (high compression)
        const LZ4HC = 3;
        /// LZHAM compression
        const LZHAM = 4;
    }
}

bitflags! {
    /// Archive flags for old Unity bundle format
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ArchiveFlagsOld: u32 {
        /// Compression type mask
        const CompressionTypeMask = 0x3F;
        /// Blocks and directory info are combined
        const BlocksAndDirectoryInfoCombined = 0x40;
        /// Blocks info is at the end
        const BlocksInfoAtTheEnd = 0x80;
        /// Old web plugin compatibility
        const OldWebPluginCompatibility = 0x100;
        /// Uses asset bundle encryption
        const UsesAssetBundleEncryption = 0x200;
    }
}

bitflags! {
    /// Archive flags for Unity bundle files
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ArchiveFlags: u32 {
        /// Compression type mask
        const CompressionTypeMask = 0x3F;
        /// Blocks and directory info are combined
        const BlocksAndDirectoryInfoCombined = 0x40;
        /// Blocks info is at the end
        const BlocksInfoAtTheEnd = 0x80;
        /// Old web plugin compatibility
        const OldWebPluginCompatibility = 0x100;
        /// Block info needs padding at start
        const BlockInfoNeedPaddingAtStart = 0x200;
        /// Uses asset bundle encryption
        const UsesAssetBundleEncryption = 0x400;
    }
}
