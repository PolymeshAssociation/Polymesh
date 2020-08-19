// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Data types and definitions of jurisdictions.

use crate::migrate::Migrate;
use codec::{Decode, Encode};
use core::str;
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// A wrapper for Jurisdiction name.
///
/// The old form of storage; deprecated.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped, Debug)]
pub struct JurisdictionName(pub Vec<u8>);

impl Migrate for JurisdictionName {
    type Into = CountryCode;
    fn migrate(self) -> Option<Self::Into> {
        str::from_utf8(&self.0).ok().and_then(CountryCode::by_any)
    }
}

macro_rules! country_codes {
    ( $([$alpha2:ident, $alpha3:ident, $un:literal, $($extra:expr),*]),* $(,)? ) => {
        /// Existing country codes according to ISO-3166-1.
        #[allow(missing_docs)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Decode, Encode)]
        #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
        #[repr(u16)]
        pub enum CountryCode {
            $($alpha2),*
        }

        impl CountryCode {
            /// Convert from `alpha-2` codes to a country code.
            pub fn by_alpha2(value: &str) -> Option<Self> {
                Some(match value {
                    $(stringify!($alpha2) => Self::$alpha2,)*
                    _ => return None,
                })
            }

            /// Convert from `alpha-3` codes to a country code.
            pub fn by_alpha3(value: &str) -> Option<Self> {
                Some(match value {
                    $(stringify!($alpha3) => Self::$alpha2,)*
                    _ => return None,
                })
            }

            /// Convert from UN codes codes to a country code.
            pub fn by_un_code(value: &str) -> Option<Self> {
                Some(match value {
                    $(stringify!($un) => Self::$alpha2,)*
                    _ => return None,
                })
            }

            /// Convert from some common names to a country code.
            /// Common names are expected to be in lower-case.
            pub fn by_common(value: &str) -> Option<Self> {
                Some(match value {
                    $($($extra => Self::$alpha2,)*)*
                    _ => return None,
                })
            }
        }
    }
}

impl CountryCode {
    /// Using heuristics, convert from a string to a country code.
    pub fn by_any(value: &str) -> Option<Self> {
        use core::str::from_utf8;

        match value.as_bytes() {
            [b'0'..=b'9', ..] => return Self::by_un_code(&value),
            [x0, x1, tail @ ..] => {
                let x0 = x0.to_ascii_uppercase();
                let x1 = x1.to_ascii_uppercase();
                match tail {
                    // Might be alpha2 (e.g., `US`) or with subdivisions, e.g., `US-FL`.
                    [] | [b'-'] => from_utf8(&[x0, x1]).ok().and_then(Self::by_alpha2),
                    [x2] => {
                        let x2 = x2.to_ascii_uppercase();
                        from_utf8(&[x0, x1, x2]).ok().and_then(Self::by_alpha3)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        .or_else(|| Self::by_common(&value.to_lowercase()))
    }
}

#[rustfmt::skip]
country_codes! (
    // [alpha2, alpha3, un_code, common names]
    [AF, AFG, 004, "afghanistan"],
    [AX, ALA, 248, "aland", "aland islands"],
    [AL, ALB, 008, "albania"],
    [DZ, DZA, 012, "algeria"],
    [AS, ASM, 016, "american samoa"],
    [AD, AND, 020, "andorra"],
    [AO, AGO, 024, "angola"],
    [AI, AIA, 660, "anguilla"],
    [AQ, ATA, 010, "antarctica"],
    [AG, ATG, 028, "antigua", "barbuda", "antigua and barbuda"],
    [AR, ARG, 032, "argentina"],
    [AM, ARM, 051, "armenia"],
    [AW, ABW, 533, "aruba"],
    [AU, AUS, 036, "australia"],
    [AT, AUT, 040, "austria"],
    [AZ, AZE, 031, "azerbaijan"],
    [BS, BHS, 044, "bahamas"],
    [BH, BHR, 048, "bahrain"],
    [BD, BGD, 050, "bangladesh"],
    [BB, BRB, 052, "barbados"],
    [BY, BLR, 112, "belarus"],
    [BE, BEL, 056, "belgium"],
    [BZ, BLZ, 084, "belize"],
    [BJ, BEN, 204, "benin"],
    [BM, BMU, 060, "bermuda"],
    [BT, BTN, 064, "bhutan"],
    [BO, BOL, 068, "bolivia"],
    [BA, BIH, 070, "bosnia", "herzegovina", "bosnia and herzegovina"],
    [BW, BWA, 072, "botswana"],
    [BV, BVT, 074, "bouvet", "bouvet island"],
    [BR, BRA, 076, "brazil"],
    [VG, VGB, 092, "british virgin islands"],
    [IO, IOT, 086, "british indian ocean territory", "indian ocean territory"],
    [BN, BRN, 096, "brunei", "darussalam", "brunei darussalam"],
    [BG, BGR, 100, "bulgaria"],
    [BF, BFA, 854, "burkina", "faso", "burkina faso"],
    [BI, BDI, 108, "burundi"],
    [KH, KHM, 116, "cambodia"],
    [CM, CMR, 120, "cameroon"],
    [CA, CAN, 124, "canada"],
    [CV, CPV, 132, "cape", "verde", "cape verde"],
    [KY, CYM, 136, "cayman", "cayman islands"],
    [CF, CAF, 140, "central african republic"],
    [TD, TCD, 148, "chad"],
    [CL, CHL, 152, "chile"],
    [CN, CHN, 156, "china"],
    [HK, HKG, 344, "hong", "hong kong", "hong Kong, sar china"],
    [MO, MAC, 446, "macao", "macao, sar china"],
    [CX, CXR, 162, "christmas", "christmas island"],
    [CC, CCK, 166, "cocos", "keeling", "cocos (keeling) islands"],
    [CO, COL, 170, "colombia"],
    [KM, COM, 174, "comoros"],
    [CG, COG, 178, "brazzaville", "congo (brazzaville)"],
    [CD, COD, 180, "kinshasa", "congo, (kinshasa)"],
    [CK, COK, 184, "cook", "cook islands"],
    [CR, CRI, 188, "costa", "costa rica"],
    [CI, CIV, 384, "ivoire", "d'ivoire", "côte", "cote", "côte d'ivoire"],
    [HR, HRV, 191, "croatia"],
    [CU, CUB, 192, "cuba"],
    [CY, CYP, 196, "cyprus"],
    [CZ, CZE, 203, "czech", "czech republic"],
    [DK, DNK, 208, "denmark"],
    [DJ, DJI, 262, "djibouti"],
    [DM, DMA, 212, "dominica"],
    [DO, DOM, 214, "dominican republic"],
    [EC, ECU, 218, "ecuador"],
    [EG, EGY, 818, "egypt"],
    [SV, SLV, 222, "salvador", "el salvador"],
    [GQ, GNQ, 226, "equatorial", "equatorial guinea"],
    [ER, ERI, 232, "eritrea"],
    [EE, EST, 233, "estonia"],
    [ET, ETH, 231, "ethiopia"],
    [FK, FLK, 238, "falkland", "falkland islands", "malvinas", "falkland islands (malvinas)"],
    [FO, FRO, 234, "faroe", "faroe islands"],
    [FJ, FJI, 242, "fiji"],
    [FI, FIN, 246, "finland"],
    [FR, FRA, 250, "france"],
    [GF, GUF, 254, "guiana", "french guiana"],
    [PF, PYF, 258, "polynesia", "french polynesia"],
    [TF, ATF, 260, "southern territories", "french southern territories"],
    [GA, GAB, 266, "gabon"],
    [GM, GMB, 270, "gambia"],
    [GE, GEO, 268, "georgia"],
    [DE, DEU, 276, "germany"],
    [GH, GHA, 288, "ghana"],
    [GI, GIB, 292, "gibraltar"],
    [GR, GRC, 300, "greece"],
    [GL, GRL, 304, "greenland"],
    [GD, GRD, 308, "grenada"],
    [GP, GLP, 312, "guadeloupe"],
    [GU, GUM, 316, "guam"],
    [GT, GTM, 320, "guatemala"],
    [GG, GGY, 831, "guernsey"],
    [GN, GIN, 324, "guinea"],
    [GW, GNB, 624, "bissau", "guinea-bissau"],
    [GY, GUY, 328, "guyana"],
    [HT, HTI, 332, "haiti"],
    [HM, HMD, 334, "heard", "mcdonald", "heard and mcdonald islands"],
    [VA, VAT, 336, "holy", "holy see", "vatican", "vatican city", "holy see (vatican city state)"],
    [HN, HND, 340, "honduras"],
    [HU, HUN, 348, "hungary"],
    [IS, ISL, 352, "iceland"],
    [IN, IND, 356, "india"],
    [ID, IDN, 360, "indonesia"],
    [IR, IRN, 364, "iran", "persia", "iran, islamic republic of"],
    [IQ, IRQ, 368, "iraq"],
    [IE, IRL, 372, "ireland"],
    [IM, IMN, 833, "isle of man"],
    [IL, ISR, 376, "israel"],
    [IT, ITA, 380, "italy"],
    [JM, JAM, 388, "jamaica"],
    [JP, JPN, 392, "japan"],
    [JE, JEY, 832, "jersey"],
    [JO, JOR, 400, "jordan"],
    [KZ, KAZ, 398, "kazakhstan"],
    [KE, KEN, 404, "kenya"],
    [KI, KIR, 296, "kiribati"],
    [KP, PRK, 408, "north korea", "korea (north)"],
    [KR, KOR, 410, "south korea", "korea (south)"],
    [KW, KWT, 414, "kuwait"],
    [KG, KGZ, 417, "kyrgyzstan"],
    [LA, LAO, 418, "lao pdr"],
    [LV, LVA, 428, "latvia"],
    [LB, LBN, 422, "lebanon"],
    [LS, LSO, 426, "lesotho"],
    [LR, LBR, 430, "liberia"],
    [LY, LBY, 434, "libya"],
    [LI, LIE, 438, "liechtenstein"],
    [LT, LTU, 440, "lithuania"],
    [LU, LUX, 442, "luxembourg"],
    [MK, MKD, 807, "macedonia", "macedonia, republic of"],
    [MG, MDG, 450, "madagascar"],
    [MW, MWI, 454, "malawi"],
    [MY, MYS, 458, "malaysia"],
    [MV, MDV, 462, "maldives"],
    [ML, MLI, 466, "mali"],
    [MT, MLT, 470, "malta"],
    [MH, MHL, 584, "marshall", "marshall islands"],
    [MQ, MTQ, 474, "martinique"],
    [MR, MRT, 478, "mauritania"],
    [MU, MUS, 480, "mauritius"],
    [YT, MYT, 175, "mayotte"],
    [MX, MEX, 484, "mexico"],
    [FM, FSM, 583, "micronesia", "micronesia, federated states of"],
    [MD, MDA, 498, "moldova"],
    [MC, MCO, 492, "monaco"],
    [MN, MNG, 496, "mongolia"],
    [ME, MNE, 499, "montenegro"],
    [MS, MSR, 500, "montserrat"],
    [MA, MAR, 504, "morocco"],
    [MZ, MOZ, 508, "mozambique"],
    [MM, MMR, 104, "myanmar"],
    [NA, NAM, 516, "namibia"],
    [NR, NRU, 520, "nauru"],
    [NP, NPL, 524, "nepal"],
    [NL, NLD, 528, "netherlands", "holland"],
    [AN, ANT, 530, "netherlands antilles"],
    [NC, NCL, 540, "new caledonia"],
    [NZ, NZL, 554, "new zealand"],
    [NI, NIC, 558, "nicaragua"],
    [NE, NER, 562, "niger"],
    [NG, NGA, 566, "nigeria"],
    [NU, NIU, 570, "niue"],
    [NF, NFK, 574, "norfolk", "norfolk island"],
    [MP, MNP, 580, "mariana", "mariana islands", "northern mariana islands"],
    [NO, NOR, 578, "norway"],
    [OM, OMN, 512, "oman"],
    [PK, PAK, 586, "pakistan"],
    [PW, PLW, 585, "palau"],
    [PS, PSE, 275, "palestine", "palestinian territory"],
    [PA, PAN, 591, "panama"],
    [PG, PNG, 598, "papua", "new guinea", "papua new guinea"],
    [PY, PRY, 600, "paraguay"],
    [PE, PER, 604, "peru"],
    [PH, PHL, 608, "philippines"],
    [PN, PCN, 612, "pitcairn"],
    [PL, POL, 616, "poland"],
    [PT, PRT, 620, "portugal"],
    [PR, PRI, 630, "puerto", "rico", "puerto rico"],
    [QA, QAT, 634, "qatar"],
    [RE, REU, 638, "reunion", "réunion"],
    [RO, ROU, 642, "romania"],
    [RU, RUS, 643, "russia", "russian", "russian federation"],
    [RW, RWA, 646, "rwanda"],
    [BL, BLM, 652, "barthelemy", "barthélemy", "saint-barthélemy", "saint-barthelemy"],
    [SH, SHN, 654, "helena", "saint helena"],
    [KN, KNA, 659, "kitts", "vevis", "saint kitts and vevis"],
    [LC, LCA, 662, "lucia", "saint lucia"],
    [MF, MAF, 663, "martin", "saint-martin", "saint-martin (french part)"],
    [PM, SPM, 666, "saint pierre", "pierre", "miquelon", "saint pierre and miquelon"],
    [VC, VCT, 670, "saint vincent", "vincent", "grenadines", "saint vincent and grenadines"],
    [WS, WSM, 882, "samoa"],
    [SM, SMR, 674, "marino", "san marino"],
    [ST, STP, 678, "tome", "sao tome", "principe", "sao tome and principe"],
    [SA, SAU, 682, "saudi", "saudi arabia"],
    [SN, SEN, 686, "senegal"],
    [RS, SRB, 688, "serbia"],
    [SC, SYC, 690, "seychelles"],
    [SL, SLE, 694, "sierra", "leone", "sierra leone"],
    [SG, SGP, 702, "singapore"],
    [SK, SVK, 703, "slovakia"],
    [SI, SVN, 705, "slovenia"],
    [SB, SLB, 090, "solomon", "solomon islands"],
    [SO, SOM, 706, "somalia"],
    [ZA, ZAF, 710, "south africa"],
    [GS, SGS, 239, "south georgia", "south sandwich islands", "south georgia and the south sandwich islands"],
    [SS, SSD, 728, "south sudan"],
    [ES, ESP, 724, "spain"],
    [LK, LKA, 144, "sri", "lanka", "sri lanka"],
    [SD, SDN, 736, "sudan"],
    [SR, SUR, 740, "suriname"],
    [SJ, SJM, 744, "svalbard", "jan mayen", "jan mayen islands", "svalbard and jan mayen islands"],
    [SZ, SWZ, 748, "swaziland"],
    [SE, SWE, 752, "sweden"],
    [CH, CHE, 756, "switzerland"],
    [SY, SYR, 760, "syria", "syrian arab republic", "syrian arab republic (syria)"],
    [TW, TWN, 158, "taiwan", "taiwan, republic of china"],
    [TJ, TJK, 762, "tajikistan"],
    [TZ, TZA, 834, "tanzania", "tanzania, united republic of"],
    [TH, THA, 764, "thailand"],
    [TL, TLS, 626, "timor", "leste", "timor-leste"],
    [TG, TGO, 768, "togo"],
    [TK, TKL, 772, "tokelau"],
    [TO, TON, 776, "tonga"],
    [TT, TTO, 780, "trinidad", "tobago", "trinidad and tobago"],
    [TN, TUN, 788, "tunisia"],
    [TR, TUR, 792, "turkey"],
    [TM, TKM, 795, "turkmenistan"],
    [TC, TCA, 796, "turks", "caicos", "turks and caicos islands"],
    [TV, TUV, 798, "tuvalu"],
    [UG, UGA, 800, "uganda"],
    [UA, UKR, 804, "ukraine"],
    [AE, ARE, 784, "emirates", "united arab emirates"],
    [GB, GBR, 826, "great britain", "england", "scotland", "united kingdom"],
    [US, USA, 840, "united states", "america", "united states of america"],
    [UM, UMI, 581, "minor outlying islands", "us minor outlying islands"],
    [UY, URY, 858, "uruguay"],
    [UZ, UZB, 860, "uzbekistan"],
    [VU, VUT, 548, "vanuatu"],
    [VE, VEN, 862, "venezuela", "bolivarian republic", "venezuela (bolivarian republic)"],
    [VN, VNM, 704, "vietnam", "viet nam"],
    [VI, VIR, 850, "virgin islands, us"],
    [WF, WLF, 876, "wallis", "futuna", "futuna islands", "wallis and futuna islands"],
    [EH, ESH, 732, "western sahara"],
    [YE, YEM, 887, "yemen"],
    [ZM, ZMB, 894, "zambia"],
    [ZW, ZWE, 716, "zimbabwe"],
);

#[cfg(test)]
#[test]
fn by_any_works() {
    const US: Option<CountryCode> = Some(CountryCode::US);
    assert_eq!(US, CountryCode::by_any("us"));
    assert_eq!(US, CountryCode::by_any("us-"));
    assert_eq!(US, CountryCode::by_any("USA"));
    assert_eq!(US, CountryCode::by_any("840"));
    assert_eq!(US, CountryCode::by_any("america"));
    assert_eq!(None, CountryCode::by_any("neverland"));
}
