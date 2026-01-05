//! Auto-generated operator implementations
//!
//! Each operator is organized into alphabetical subfolders.

use super::operator_data::OperatorData;
use super::operator_unit::{DpsCalculator, OperatorParams};

pub mod a;
pub mod b;
pub mod c;
pub mod d;
pub mod e;
pub mod f;
pub mod g;
pub mod h;
pub mod i;
pub mod j;
pub mod k;
pub mod l;
pub mod m;
pub mod n;
pub mod o;
pub mod p;
pub mod q;
pub mod r;
pub mod s;
pub mod t;
pub mod u;
pub mod v;
pub mod w;
pub mod y;
pub mod z;

pub use a::*;
pub use b::*;
pub use c::*;
pub use d::*;
pub use e::*;
pub use f::*;
pub use g::*;
pub use h::*;
pub use i::*;
pub use j::*;
pub use k::*;
pub use l::*;
pub use m::*;
pub use n::*;
pub use o::*;
pub use p::*;
pub use q::*;
pub use r::*;
pub use s::*;
pub use t::*;
pub use u::*;
pub use v::*;
pub use w::*;
pub use y::*;
pub use z::*;

/// Creates an operator by name, returning a boxed DpsCalculator trait object
///
/// # Arguments
/// * `name` - The operator name (e.g., "Blaze", "ExusiaiAlter")
/// * `operator_data` - The operator's base data from JSON
/// * `params` - The operator configuration parameters
///
/// # Returns
/// Some(Box<dyn DpsCalculator>) if the operator is found, None otherwise
pub fn create_operator(
    name: &str,
    operator_data: OperatorData,
    params: OperatorParams,
) -> Option<Box<dyn DpsCalculator + Send + Sync>> {
    match name {
        "Aak" => Some(Box::new(Aak::new(operator_data, params))),
        "Absinthe" => Some(Box::new(Absinthe::new(operator_data, params))),
        "Aciddrop" => Some(Box::new(Aciddrop::new(operator_data, params))),
        "Adnachiel" => Some(Box::new(Adnachiel::new(operator_data, params))),
        "Amiya" => Some(Box::new(Amiya::new(operator_data, params))),
        "AmiyaGuard" => Some(Box::new(AmiyaGuard::new(operator_data, params))),
        "AmiyaMedic" => Some(Box::new(AmiyaMedic::new(operator_data, params))),
        "Andreana" => Some(Box::new(Andreana::new(operator_data, params))),
        "Angelina" => Some(Box::new(Angelina::new(operator_data, params))),
        "Aosta" => Some(Box::new(Aosta::new(operator_data, params))),
        "April" => Some(Box::new(April::new(operator_data, params))),
        "Archetto" => Some(Box::new(Archetto::new(operator_data, params))),
        "Arene" => Some(Box::new(Arene::new(operator_data, params))),
        "Asbestos" => Some(Box::new(Asbestos::new(operator_data, params))),
        "Ascalon" => Some(Box::new(Ascalon::new(operator_data, params))),
        "Ash" => Some(Box::new(Ash::new(operator_data, params))),
        "Ashlock" => Some(Box::new(Ashlock::new(operator_data, params))),
        "Astesia" => Some(Box::new(Astesia::new(operator_data, params))),
        "Astgenne" => Some(Box::new(Astgenne::new(operator_data, params))),
        "Aurora" => Some(Box::new(Aurora::new(operator_data, params))),
        "Ayerscarpe" => Some(Box::new(Ayerscarpe::new(operator_data, params))),
        "Bagpipe" => Some(Box::new(Bagpipe::new(operator_data, params))),
        "Beehunter" => Some(Box::new(Beehunter::new(operator_data, params))),
        "Beeswax" => Some(Box::new(Beeswax::new(operator_data, params))),
        "Bibeak" => Some(Box::new(Bibeak::new(operator_data, params))),
        "Blaze" => Some(Box::new(Blaze::new(operator_data, params))),
        "BlazeAlter" => Some(Box::new(BlazeAlter::new(operator_data, params))),
        "Blemishine" => Some(Box::new(Blemishine::new(operator_data, params))),
        "Blitz" => Some(Box::new(Blitz::new(operator_data, params))),
        "BluePoison" => Some(Box::new(BluePoison::new(operator_data, params))),
        "Broca" => Some(Box::new(Broca::new(operator_data, params))),
        "Bryophyta" => Some(Box::new(Bryophyta::new(operator_data, params))),
        "Cantabile" => Some(Box::new(Cantabile::new(operator_data, params))),
        "Caper" => Some(Box::new(Caper::new(operator_data, params))),
        "Carnelian" => Some(Box::new(Carnelian::new(operator_data, params))),
        "Castle3" => Some(Box::new(Castle3::new(operator_data, params))),
        "Catapult" => Some(Box::new(Catapult::new(operator_data, params))),
        "Ceobe" => Some(Box::new(Ceobe::new(operator_data, params))),
        "Chen" => Some(Box::new(Chen::new(operator_data, params))),
        "ChenAlter" => Some(Box::new(ChenAlter::new(operator_data, params))),
        "Chongyue" => Some(Box::new(Chongyue::new(operator_data, params))),
        "CivilightEterna" => Some(Box::new(CivilightEterna::new(operator_data, params))),
        "Click" => Some(Box::new(Click::new(operator_data, params))),
        "Coldshot" => Some(Box::new(Coldshot::new(operator_data, params))),
        "Contrail" => Some(Box::new(Contrail::new(operator_data, params))),
        "Conviction" => Some(Box::new(Conviction::new(operator_data, params))),
        "Crownslayer" => Some(Box::new(Crownslayer::new(operator_data, params))),
        "Dagda" => Some(Box::new(Dagda::new(operator_data, params))),
        "Degenbrecher" => Some(Box::new(Degenbrecher::new(operator_data, params))),
        "Diamante" => Some(Box::new(Diamante::new(operator_data, params))),
        "Dobermann" => Some(Box::new(Dobermann::new(operator_data, params))),
        "Doc" => Some(Box::new(Doc::new(operator_data, params))),
        "Dorothy" => Some(Box::new(Dorothy::new(operator_data, params))),
        "Durin" => Some(Box::new(Durin::new(operator_data, params))),
        "Durnar" => Some(Box::new(Durnar::new(operator_data, params))),
        "Dusk" => Some(Box::new(Dusk::new(operator_data, params))),
        "Ebenholz" => Some(Box::new(Ebenholz::new(operator_data, params))),
        "Ela" => Some(Box::new(Ela::new(operator_data, params))),
        "Entelechia" => Some(Box::new(Entelechia::new(operator_data, params))),
        "Erato" => Some(Box::new(Erato::new(operator_data, params))),
        "Estelle" => Some(Box::new(Estelle::new(operator_data, params))),
        "Ethan" => Some(Box::new(Ethan::new(operator_data, params))),
        "Eunectes" => Some(Box::new(Eunectes::new(operator_data, params))),
        "ExecutorAlter" => Some(Box::new(ExecutorAlter::new(operator_data, params))),
        "Exusiai" => Some(Box::new(Exusiai::new(operator_data, params))),
        "ExusiaiAlter" => Some(Box::new(ExusiaiAlter::new(operator_data, params))),
        "Eyjafjalla" => Some(Box::new(Eyjafjalla::new(operator_data, params))),
        "FangAlter" => Some(Box::new(FangAlter::new(operator_data, params))),
        "Fartooth" => Some(Box::new(Fartooth::new(operator_data, params))),
        "Fiammetta" => Some(Box::new(Fiammetta::new(operator_data, params))),
        "Figurino" => Some(Box::new(Figurino::new(operator_data, params))),
        "Firewhistle" => Some(Box::new(Firewhistle::new(operator_data, params))),
        "Flamebringer" => Some(Box::new(Flamebringer::new(operator_data, params))),
        "Flametail" => Some(Box::new(Flametail::new(operator_data, params))),
        "Flint" => Some(Box::new(Flint::new(operator_data, params))),
        "Folinic" => Some(Box::new(Folinic::new(operator_data, params))),
        "Franka" => Some(Box::new(Franka::new(operator_data, params))),
        "Frost" => Some(Box::new(Frost::new(operator_data, params))),
        "Frostleaf" => Some(Box::new(Frostleaf::new(operator_data, params))),
        "Fuze" => Some(Box::new(Fuze::new(operator_data, params))),
        "GavialAlter" => Some(Box::new(GavialAlter::new(operator_data, params))),
        "Gladiia" => Some(Box::new(Gladiia::new(operator_data, params))),
        "Gnosis" => Some(Box::new(Gnosis::new(operator_data, params))),
        "Goldenglow" => Some(Box::new(Goldenglow::new(operator_data, params))),
        "Gracebearer" => Some(Box::new(Gracebearer::new(operator_data, params))),
        "Grani" => Some(Box::new(Grani::new(operator_data, params))),
        "GreyThroat" => Some(Box::new(GreyThroat::new(operator_data, params))),
        "GreyyAlter" => Some(Box::new(GreyyAlter::new(operator_data, params))),
        "Hadiya" => Some(Box::new(Hadiya::new(operator_data, params))),
        "Harmonie" => Some(Box::new(Harmonie::new(operator_data, params))),
        "Haze" => Some(Box::new(Haze::new(operator_data, params))),
        "Hellagur" => Some(Box::new(Hellagur::new(operator_data, params))),
        "Hibiscus" => Some(Box::new(Hibiscus::new(operator_data, params))),
        "Highmore" => Some(Box::new(Highmore::new(operator_data, params))),
        "Hoederer" => Some(Box::new(Hoederer::new(operator_data, params))),
        "Hoolheyak" => Some(Box::new(Hoolheyak::new(operator_data, params))),
        "Horn" => Some(Box::new(Horn::new(operator_data, params))),
        "Hoshiguma" => Some(Box::new(Hoshiguma::new(operator_data, params))),
        "HoshigumaAlter" => Some(Box::new(HoshigumaAlter::new(operator_data, params))),
        "Humus" => Some(Box::new(Humus::new(operator_data, params))),
        "Iana" => Some(Box::new(Iana::new(operator_data, params))),
        "Ifrit" => Some(Box::new(Ifrit::new(operator_data, params))),
        "Indra" => Some(Box::new(Indra::new(operator_data, params))),
        "Ines" => Some(Box::new(Ines::new(operator_data, params))),
        "Insider" => Some(Box::new(Insider::new(operator_data, params))),
        "Irene" => Some(Box::new(Irene::new(operator_data, params))),
        "Jackie" => Some(Box::new(Jackie::new(operator_data, params))),
        "Jaye" => Some(Box::new(Jaye::new(operator_data, params))),
        "Jessica" => Some(Box::new(Jessica::new(operator_data, params))),
        "JessicaAlter" => Some(Box::new(JessicaAlter::new(operator_data, params))),
        "JusticeKnight" => Some(Box::new(JusticeKnight::new(operator_data, params))),
        "Kafka" => Some(Box::new(Kafka::new(operator_data, params))),
        "Kaltsit" => Some(Box::new(Kaltsit::new(operator_data, params))),
        "Kazemaru" => Some(Box::new(Kazemaru::new(operator_data, params))),
        "Kirara" => Some(Box::new(Kirara::new(operator_data, params))),
        "Kjera" => Some(Box::new(Kjera::new(operator_data, params))),
        "Kroos" => Some(Box::new(Kroos::new(operator_data, params))),
        "KroosAlter" => Some(Box::new(KroosAlter::new(operator_data, params))),
        "LaPluma" => Some(Box::new(LaPluma::new(operator_data, params))),
        "Laios" => Some(Box::new(Laios::new(operator_data, params))),
        "Lappland" => Some(Box::new(Lappland::new(operator_data, params))),
        "LapplandAlter" => Some(Box::new(LapplandAlter::new(operator_data, params))),
        "Lava3star" => Some(Box::new(Lava3star::new(operator_data, params))),
        "Lavaalt" => Some(Box::new(Lavaalt::new(operator_data, params))),
        "Lee" => Some(Box::new(Lee::new(operator_data, params))),
        "LeiziAlter" => Some(Box::new(LeiziAlter::new(operator_data, params))),
        "Lemuen" => Some(Box::new(Lemuen::new(operator_data, params))),
        "Lessing" => Some(Box::new(Lessing::new(operator_data, params))),
        "Leto" => Some(Box::new(Leto::new(operator_data, params))),
        "Lin" => Some(Box::new(Lin::new(operator_data, params))),
        "Ling" => Some(Box::new(Ling::new(operator_data, params))),
        "Logos" => Some(Box::new(Logos::new(operator_data, params))),
        "Lucilla" => Some(Box::new(Lucilla::new(operator_data, params))),
        "Lunacub" => Some(Box::new(Lunacub::new(operator_data, params))),
        "LuoXiaohei" => Some(Box::new(LuoXiaohei::new(operator_data, params))),
        "Lutonada" => Some(Box::new(Lutonada::new(operator_data, params))),
        "Magallan" => Some(Box::new(Magallan::new(operator_data, params))),
        "Manticore" => Some(Box::new(Manticore::new(operator_data, params))),
        "Marcille" => Some(Box::new(Marcille::new(operator_data, params))),
        "Matoimaru" => Some(Box::new(Matoimaru::new(operator_data, params))),
        "May" => Some(Box::new(May::new(operator_data, params))),
        "Melantha" => Some(Box::new(Melantha::new(operator_data, params))),
        "Meteor" => Some(Box::new(Meteor::new(operator_data, params))),
        "Meteorite" => Some(Box::new(Meteorite::new(operator_data, params))),
        "Midnight" => Some(Box::new(Midnight::new(operator_data, params))),
        "Minimalist" => Some(Box::new(Minimalist::new(operator_data, params))),
        "Mint" => Some(Box::new(Mint::new(operator_data, params))),
        "MissChristine" => Some(Box::new(MissChristine::new(operator_data, params))),
        "MisumiUika" => Some(Box::new(MisumiUika::new(operator_data, params))),
        "Mizuki" => Some(Box::new(Mizuki::new(operator_data, params))),
        "Mlynar" => Some(Box::new(Mlynar::new(operator_data, params))),
        "Mon3tr" => Some(Box::new(Mon3tr::new(operator_data, params))),
        "Morgan" => Some(Box::new(Morgan::new(operator_data, params))),
        "Mostima" => Some(Box::new(Mostima::new(operator_data, params))),
        "Mountain" => Some(Box::new(Mountain::new(operator_data, params))),
        "Mousse" => Some(Box::new(Mousse::new(operator_data, params))),
        "MrNothing" => Some(Box::new(MrNothing::new(operator_data, params))),
        "Mudrock" => Some(Box::new(Mudrock::new(operator_data, params))),
        "Muelsyse" => Some(Box::new(Muelsyse::new(operator_data, params))),
        "Narantuya" => Some(Box::new(Narantuya::new(operator_data, params))),
        "NearlAlter" => Some(Box::new(NearlAlter::new(operator_data, params))),
        "Necrass" => Some(Box::new(Necrass::new(operator_data, params))),
        "Nian" => Some(Box::new(Nian::new(operator_data, params))),
        "Nymph" => Some(Box::new(Nymph::new(operator_data, params))),
        "Odda" => Some(Box::new(Odda::new(operator_data, params))),
        "Pallas" => Some(Box::new(Pallas::new(operator_data, params))),
        "Passenger" => Some(Box::new(Passenger::new(operator_data, params))),
        "Penance" => Some(Box::new(Penance::new(operator_data, params))),
        "Pepe" => Some(Box::new(Pepe::new(operator_data, params))),
        "Phantom" => Some(Box::new(Phantom::new(operator_data, params))),
        "Pinecone" => Some(Box::new(Pinecone::new(operator_data, params))),
        "Pith" => Some(Box::new(Pith::new(operator_data, params))),
        "Platinum" => Some(Box::new(Platinum::new(operator_data, params))),
        "Plume" => Some(Box::new(Plume::new(operator_data, params))),
        "Popukar" => Some(Box::new(Popukar::new(operator_data, params))),
        "Pozemka" => Some(Box::new(Pozemka::new(operator_data, params))),
        "PramanixAlter" => Some(Box::new(PramanixAlter::new(operator_data, params))),
        "ProjektRed" => Some(Box::new(ProjektRed::new(operator_data, params))),
        "Provence" => Some(Box::new(Provence::new(operator_data, params))),
        "Pudding" => Some(Box::new(Pudding::new(operator_data, params))),
        "Qiubai" => Some(Box::new(Qiubai::new(operator_data, params))),
        "Quartz" => Some(Box::new(Quartz::new(operator_data, params))),
        "Raidian" => Some(Box::new(Raidian::new(operator_data, params))),
        "Rangers" => Some(Box::new(Rangers::new(operator_data, params))),
        "Ray" => Some(Box::new(Ray::new(operator_data, params))),
        "ReedAlter" => Some(Box::new(ReedAlter::new(operator_data, params))),
        "Rockrock" => Some(Box::new(Rockrock::new(operator_data, params))),
        "Rosa" => Some(Box::new(Rosa::new(operator_data, params))),
        "Rosmontis" => Some(Box::new(Rosmontis::new(operator_data, params))),
        "Saga" => Some(Box::new(Saga::new(operator_data, params))),
        "SandReckoner" => Some(Box::new(SandReckoner::new(operator_data, params))),
        "SanktaMiksaparato" => Some(Box::new(SanktaMiksaparato::new(operator_data, params))),
        "Savage" => Some(Box::new(Savage::new(operator_data, params))),
        "Scavenger" => Some(Box::new(Scavenger::new(operator_data, params))),
        "Scene" => Some(Box::new(Scene::new(operator_data, params))),
        "Schwarz" => Some(Box::new(Schwarz::new(operator_data, params))),
        "Shalem" => Some(Box::new(Shalem::new(operator_data, params))),
        "Sharp" => Some(Box::new(Sharp::new(operator_data, params))),
        "Sideroca" => Some(Box::new(Sideroca::new(operator_data, params))),
        "Siege" => Some(Box::new(Siege::new(operator_data, params))),
        "SilverAsh" => Some(Box::new(SilverAsh::new(operator_data, params))),
        "Skadi" => Some(Box::new(Skadi::new(operator_data, params))),
        "Skalter" => Some(Box::new(Skalter::new(operator_data, params))),
        "Snegurochka" => Some(Box::new(Snegurochka::new(operator_data, params))),
        "Specter" => Some(Box::new(Specter::new(operator_data, params))),
        "SpecterAlter" => Some(Box::new(SpecterAlter::new(operator_data, params))),
        "Stainless" => Some(Box::new(Stainless::new(operator_data, params))),
        "Steward" => Some(Box::new(Steward::new(operator_data, params))),
        "Stormeye" => Some(Box::new(Stormeye::new(operator_data, params))),
        "Surfer" => Some(Box::new(Surfer::new(operator_data, params))),
        "Surtr" => Some(Box::new(Surtr::new(operator_data, params))),
        "Suzuran" => Some(Box::new(Suzuran::new(operator_data, params))),
        "SwireAlt" => Some(Box::new(SwireAlt::new(operator_data, params))),
        "Tachanka" => Some(Box::new(Tachanka::new(operator_data, params))),
        "Tecno" => Some(Box::new(Tecno::new(operator_data, params))),
        "Tequila" => Some(Box::new(Tequila::new(operator_data, params))),
        "TerraResearchCommission" => Some(Box::new(TerraResearchCommission::new(
            operator_data,
            params,
        ))),
        "TexasAlter" => Some(Box::new(TexasAlter::new(operator_data, params))),
        "Thorns" => Some(Box::new(Thorns::new(operator_data, params))),
        "ThornsAlter" => Some(Box::new(ThornsAlter::new(operator_data, params))),
        "TinMan" => Some(Box::new(TinMan::new(operator_data, params))),
        "Tippi" => Some(Box::new(Tippi::new(operator_data, params))),
        "Toddifons" => Some(Box::new(Toddifons::new(operator_data, params))),
        "TogawaSakiko" => Some(Box::new(TogawaSakiko::new(operator_data, params))),
        "Tomimi" => Some(Box::new(Tomimi::new(operator_data, params))),
        "Totter" => Some(Box::new(Totter::new(operator_data, params))),
        "Tragodia" => Some(Box::new(Tragodia::new(operator_data, params))),
        "Typhon" => Some(Box::new(Typhon::new(operator_data, params))),
        "Ulpianus" => Some(Box::new(Ulpianus::new(operator_data, params))),
        "Underflow" => Some(Box::new(Underflow::new(operator_data, params))),
        "Utage" => Some(Box::new(Utage::new(operator_data, params))),
        "Vanilla" => Some(Box::new(Vanilla::new(operator_data, params))),
        "Vendela" => Some(Box::new(Vendela::new(operator_data, params))),
        "Vermeil" => Some(Box::new(Vermeil::new(operator_data, params))),
        "Vetochki" => Some(Box::new(Vetochki::new(operator_data, params))),
        "Vigil" => Some(Box::new(Vigil::new(operator_data, params))),
        "Vigna" => Some(Box::new(Vigna::new(operator_data, params))),
        "Vina" => Some(Box::new(Vina::new(operator_data, params))),
        "Virtuosa" => Some(Box::new(Virtuosa::new(operator_data, params))),
        "Viviana" => Some(Box::new(Viviana::new(operator_data, params))),
        "Vulcan" => Some(Box::new(Vulcan::new(operator_data, params))),
        "Vulpisfoglia" => Some(Box::new(Vulpisfoglia::new(operator_data, params))),
        "W" => Some(Box::new(W::new(operator_data, params))),
        "WakabaMutsumi" => Some(Box::new(WakabaMutsumi::new(operator_data, params))),
        "Walter" => Some(Box::new(Walter::new(operator_data, params))),
        "Warmy" => Some(Box::new(Warmy::new(operator_data, params))),
        "Weedy" => Some(Box::new(Weedy::new(operator_data, params))),
        "Whislash" => Some(Box::new(Whislash::new(operator_data, params))),
        "Wildmane" => Some(Box::new(Wildmane::new(operator_data, params))),
        "Windscoot" => Some(Box::new(Windscoot::new(operator_data, params))),
        "YahataUmiri" => Some(Box::new(YahataUmiri::new(operator_data, params))),
        "YatoAlter" => Some(Box::new(YatoAlter::new(operator_data, params))),
        "Yu" => Some(Box::new(Yu::new(operator_data, params))),
        "YutenjiNyamu" => Some(Box::new(YutenjiNyamu::new(operator_data, params))),
        "ZuoLe" => Some(Box::new(ZuoLe::new(operator_data, params))),
        "TwelveF" => Some(Box::new(TwelveF::new(operator_data, params))),
        _ => None,
    }
}
