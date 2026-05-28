//! Auto-generated dispatch table. DO NOT EDIT MANUALLY.
//! Sibling of `generated_hps.rs` — both are declared by the hand-written `mod.rs`.

#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

use super::super::engine::HpsResult;
use super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::generated_hps;

/// Dispatch the operator's healing formula. Returns None when the
/// operator has no transpiled HPS implementation.
pub fn dispatch(unit: &OperatorUnit) -> Option<HpsResult> {
    // Most healers don't need enemy stats; pass a zero target so the
    // generated functions' boilerplate `defense`/`res` locals stay 0.
    let enemy = EnemyStats {
        defense: 0.0,
        res: 0.0,
    };
    let op_id = unit.data.data.id.as_deref().unwrap_or("");
    match op_id {
        "char_1037_amiya3" => generated_hps::amiya_medic(unit, &enemy),
        "char_212_ansel" => generated_hps::ansel(unit, &enemy),
        "char_4109_baslin" => generated_hps::bassline(unit, &enemy),
        "char_423_blemsh" => generated_hps::blemishine(unit, &enemy),
        "char_275_breeze" => generated_hps::breeze(unit, &enemy),
        "char_348_ceylon" => generated_hps::ceylon(unit, &enemy),
        "char_4041_chnut" => generated_hps::chestnut(unit, &enemy),
        "char_4134_cetsyr" => generated_hps::civilight_eterna(unit, &enemy),
        "char_4125_rdoc" => generated_hps::doc(unit, &enemy),
        "char_1016_agoat2" => generated_hps::eyjaberry(unit, &enemy),
        "char_345_folnic" => generated_hps::folinic(unit, &enemy),
        "char_187_ccheal" => generated_hps::gavial(unit, &enemy),
        "char_196_sunbr" => generated_hps::gummy(unit, &enemy),
        "char_4114_harold" => generated_hps::harold(unit, &enemy),
        "char_4202_haruka" => generated_hps::haruka(unit, &enemy),
        "char_4045_heidi" => generated_hps::heidi(unit, &enemy),
        "char_120_hibisc" => generated_hps::hibiscus(unit, &enemy),
        "char_449_glider" => generated_hps::honeyberry(unit, &enemy),
        "char_226_hmau" => generated_hps::hung(unit, &enemy),
        "char_003_kalts" => generated_hps::kaltsit(unit, &enemy),
        "char_285_medic2" => generated_hps::lancet2(unit, &enemy),
        "char_4042_lumen" => generated_hps::lumen(unit, &enemy),
        "char_4179_monstr" => generated_hps::mon3tr(unit, &enemy),
        "char_473_mberry" => generated_hps::mulberry(unit, &enemy),
        "char_117_myrrh" => generated_hps::myrrh(unit, &enemy),
        "char_151_myrtle" => generated_hps::myrtle(unit, &enemy),
        "char_148_nearl" => generated_hps::nearl(unit, &enemy),
        "char_179_cgbird" => generated_hps::nightingale(unit, &enemy),
        "char_164_nightm" => generated_hps::nightmare(unit, &enemy),
        "char_4019_ncdeer" => generated_hps::nine_colored_deer(unit, &enemy),
        "char_4173_nowell" => generated_hps::nowell(unit, &enemy),
        "char_4071_peper" => generated_hps::paprika(unit, &enemy),
        "char_4139_papyrs" => generated_hps::papyrus(unit, &enemy),
        "char_181_flower" => generated_hps::perfumer(unit, &enemy),
        "char_258_podego" => generated_hps::podenco(unit, &enemy),
        "char_128_plosis" => generated_hps::ptilopsis(unit, &enemy),
        "char_385_finlpp" => generated_hps::purestream(unit, &enemy),
        "char_492_quercu" => generated_hps::quercus(unit, &enemy),
        "char_4196_reckpr" => generated_hps::record_keeper(unit, &enemy),
        "char_1020_reed2" => generated_hps::reed_alter(unit, &enemy),
        "char_4163_rosesa" => generated_hps::rose_salt(unit, &enemy),
        "char_479_sleach" => generated_hps::saileach(unit, &enemy),
        "char_202_demkni" => generated_hps::saria(unit, &enemy),
        "char_4143_sensi" => generated_hps::senshi(unit, &enemy),
        "char_147_shining" => generated_hps::shining(unit, &enemy),
        "char_2025_shu" => generated_hps::shu(unit, &enemy),
        "char_108_silent" => generated_hps::silence(unit, &enemy),
        "char_1031_slent2" => generated_hps::silence_alter(unit, &enemy),
        "char_1012_skadi2" => generated_hps::skalter(unit, &enemy),
        "char_101_sora" => generated_hps::sora(unit, &enemy),
        "char_284_spot" => generated_hps::spot(unit, &enemy),
        "char_298_susuro" => generated_hps::sussurro(unit, &enemy),
        "char_1033_swire2" => generated_hps::swire_alter(unit, &enemy),
        "char_1039_thorn2" => generated_hps::thorns_alter(unit, &enemy),
        "char_343_tknogi" => generated_hps::tsukinogi(unit, &enemy),
        "char_402_tuye" => generated_hps::tuye(unit, &enemy),
        "char_4091_ulika" => generated_hps::u_official(unit, &enemy),
        "char_4119_wanqin" => generated_hps::wanqing(unit, &enemy),
        "char_171_bldsk" => generated_hps::warfarin(unit, &enemy),
        "char_436_whispr" => generated_hps::whisperain(unit, &enemy),
        "char_4172_xingzh" => generated_hps::xingzhu(unit, &enemy),
        _ => None,
    }
}
