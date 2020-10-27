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

use crate::migrate::{Empty, Migrate};
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
    type Context = Empty;
    fn migrate(self, _: Self::Context) -> Option<Self::Into> {
        str::from_utf8(&self.0).ok().and_then(CountryCode::by_any)
    }
}

macro_rules! country_codes {
    ( $([$discr:expr,$alpha2:ident, $alpha3:ident, $un:literal, $($extra:expr),*]),* $(,)? ) => {
        /// Existing country codes according to ISO-3166-1.
        #[allow(missing_docs)]
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Decode, Encode)]
        #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
        #[repr(u16)] // Could use `u8`, strictly speaking, but leave room for growth!
        pub enum CountryCode {
            $($alpha2 = $discr),*
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
    // [discriminant, alpha2, alpha3, un_code, common names]
    [1, AF, AFG, 004, "afghanistan"],
    [2, AX, ALA, 248, "aland", "aland islands"],
    [3, AL, ALB, 008, "albania"],
    [4, DZ, DZA, 012, "algeria"],
    [5, AS, ASM, 016, "american samoa"],
    [6, AD, AND, 020, "andorra"],
    [7, AO, AGO, 024, "angola"],
    [8, AI, AIA, 660, "anguilla"],
    [9, AQ, ATA, 010, "antarctica"],
    [10, AG, ATG, 028, "antigua", "barbuda", "antigua and barbuda"],
    [11, AR, ARG, 032, "argentina"],
    [12, AM, ARM, 051, "armenia"],
    [13, AW, ABW, 533, "aruba"],
    [14, AU, AUS, 036, "australia"],
    [15, AT, AUT, 040, "austria"],
    [16, AZ, AZE, 031, "azerbaijan"],
    [17, BS, BHS, 044, "bahamas"],
    [18, BH, BHR, 048, "bahrain"],
    [19, BD, BGD, 050, "bangladesh"],
    [20, BB, BRB, 052, "barbados"],
    [21, BY, BLR, 112, "belarus"],
    [22, BE, BEL, 056, "belgium"],
    [23, BZ, BLZ, 084, "belize"],
    [24, BJ, BEN, 204, "benin"],
    [25, BM, BMU, 060, "bermuda"],
    [26, BT, BTN, 064, "bhutan"],
    [27, BO, BOL, 068, "bolivia"],
    [28, BA, BIH, 070, "bosnia", "herzegovina", "bosnia and herzegovina"],
    [29, BW, BWA, 072, "botswana"],
    [30, BV, BVT, 074, "bouvet", "bouvet island"],
    [31, BR, BRA, 076, "brazil"],
    [32, VG, VGB, 092, "british virgin islands"],
    [33, IO, IOT, 086, "british indian ocean territory", "indian ocean territory"],
    [34, BN, BRN, 096, "brunei", "darussalam", "brunei darussalam"],
    [35, BG, BGR, 100, "bulgaria"],
    [36, BF, BFA, 854, "burkina", "faso", "burkina faso"],
    [37, BI, BDI, 108, "burundi"],
    [38, KH, KHM, 116, "cambodia"],
    [39, CM, CMR, 120, "cameroon"],
    [40, CA, CAN, 124, "canada"],
    [41, CV, CPV, 132, "cape", "verde", "cape verde"],
    [42, KY, CYM, 136, "cayman", "cayman islands"],
    [43, CF, CAF, 140, "central african republic"],
    [44, TD, TCD, 148, "chad"],
    [45, CL, CHL, 152, "chile"],
    [46, CN, CHN, 156, "china"],
    [47, HK, HKG, 344, "hong", "hong kong", "hong Kong, sar china"],
    [48, MO, MAC, 446, "macao", "macao, sar china"],
    [49, CX, CXR, 162, "christmas", "christmas island"],
    [50, CC, CCK, 166, "cocos", "keeling", "cocos (keeling) islands"],
    [51, CO, COL, 170, "colombia"],
    [52, KM, COM, 174, "comoros"],
    [53, CG, COG, 178, "brazzaville", "congo (brazzaville)"],
    [54, CD, COD, 180, "kinshasa", "congo, (kinshasa)"],
    [55, CK, COK, 184, "cook", "cook islands"],
    [56, CR, CRI, 188, "costa", "costa rica"],
    [57, CI, CIV, 384, "ivoire", "d'ivoire", "côte", "cote", "côte d'ivoire"],
    [58, HR, HRV, 191, "croatia"],
    [59, CU, CUB, 192, "cuba"],
    [60, CY, CYP, 196, "cyprus"],
    [61, CZ, CZE, 203, "czech", "czech republic"],
    [62, DK, DNK, 208, "denmark"],
    [63, DJ, DJI, 262, "djibouti"],
    [64, DM, DMA, 212, "dominica"],
    [65, DO, DOM, 214, "dominican republic"],
    [66, EC, ECU, 218, "ecuador"],
    [67, EG, EGY, 818, "egypt"],
    [68, SV, SLV, 222, "salvador", "el salvador"],
    [69, GQ, GNQ, 226, "equatorial", "equatorial guinea"],
    [70, ER, ERI, 232, "eritrea"],
    [71, EE, EST, 233, "estonia"],
    [72, ET, ETH, 231, "ethiopia"],
    [73, FK, FLK, 238, "falkland", "falkland islands", "malvinas", "falkland islands (malvinas)"],
    [74, FO, FRO, 234, "faroe", "faroe islands"],
    [75, FJ, FJI, 242, "fiji"],
    [76, FI, FIN, 246, "finland"],
    [77, FR, FRA, 250, "france"],
    [78, GF, GUF, 254, "guiana", "french guiana"],
    [79, PF, PYF, 258, "polynesia", "french polynesia"],
    [80, TF, ATF, 260, "southern territories", "french southern territories"],
    [81, GA, GAB, 266, "gabon"],
    [82, GM, GMB, 270, "gambia"],
    [83, GE, GEO, 268, "georgia"],
    [84, DE, DEU, 276, "germany"],
    [85, GH, GHA, 288, "ghana"],
    [86, GI, GIB, 292, "gibraltar"],
    [87, GR, GRC, 300, "greece"],
    [88, GL, GRL, 304, "greenland"],
    [89, GD, GRD, 308, "grenada"],
    [90, GP, GLP, 312, "guadeloupe"],
    [91, GU, GUM, 316, "guam"],
    [92, GT, GTM, 320, "guatemala"],
    [93, GG, GGY, 831, "guernsey"],
    [94, GN, GIN, 324, "guinea"],
    [95, GW, GNB, 624, "bissau", "guinea-bissau"],
    [96, GY, GUY, 328, "guyana"],
    [97, HT, HTI, 332, "haiti"],
    [98, HM, HMD, 334, "heard", "mcdonald", "heard and mcdonald islands"],
    [99, VA, VAT, 336, "holy", "holy see", "vatican", "vatican city", "holy see (vatican city state)"],
    [100, HN, HND, 340, "honduras"],
    [101, HU, HUN, 348, "hungary"],
    [102, IS, ISL, 352, "iceland"],
    [103, IN, IND, 356, "india"],
    [104, ID, IDN, 360, "indonesia"],
    [105, IR, IRN, 364, "iran", "persia", "iran, islamic republic of"],
    [106, IQ, IRQ, 368, "iraq"],
    [107, IE, IRL, 372, "ireland"],
    [108, IM, IMN, 833, "isle of man"],
    [109, IL, ISR, 376, "israel"],
    [110, IT, ITA, 380, "italy"],
    [111, JM, JAM, 388, "jamaica"],
    [112, JP, JPN, 392, "japan"],
    [113, JE, JEY, 832, "jersey"],
    [114, JO, JOR, 400, "jordan"],
    [115, KZ, KAZ, 398, "kazakhstan"],
    [116, KE, KEN, 404, "kenya"],
    [117, KI, KIR, 296, "kiribati"],
    [118, KP, PRK, 408, "north korea", "korea (north)"],
    [119, KR, KOR, 410, "south korea", "korea (south)"],
    [120, KW, KWT, 414, "kuwait"],
    [121, KG, KGZ, 417, "kyrgyzstan"],
    [122, LA, LAO, 418, "lao pdr"],
    [123, LV, LVA, 428, "latvia"],
    [124, LB, LBN, 422, "lebanon"],
    [125, LS, LSO, 426, "lesotho"],
    [126, LR, LBR, 430, "liberia"],
    [127, LY, LBY, 434, "libya"],
    [128, LI, LIE, 438, "liechtenstein"],
    [129, LT, LTU, 440, "lithuania"],
    [130, LU, LUX, 442, "luxembourg"],
    [131, MK, MKD, 807, "macedonia", "macedonia, republic of"],
    [132, MG, MDG, 450, "madagascar"],
    [133, MW, MWI, 454, "malawi"],
    [134, MY, MYS, 458, "malaysia"],
    [135, MV, MDV, 462, "maldives"],
    [136, ML, MLI, 466, "mali"],
    [137, MT, MLT, 470, "malta"],
    [138, MH, MHL, 584, "marshall", "marshall islands"],
    [139, MQ, MTQ, 474, "martinique"],
    [140, MR, MRT, 478, "mauritania"],
    [141, MU, MUS, 480, "mauritius"],
    [142, YT, MYT, 175, "mayotte"],
    [143, MX, MEX, 484, "mexico"],
    [144, FM, FSM, 583, "micronesia", "micronesia, federated states of"],
    [145, MD, MDA, 498, "moldova"],
    [146, MC, MCO, 492, "monaco"],
    [147, MN, MNG, 496, "mongolia"],
    [148, ME, MNE, 499, "montenegro"],
    [149, MS, MSR, 500, "montserrat"],
    [150, MA, MAR, 504, "morocco"],
    [151, MZ, MOZ, 508, "mozambique"],
    [152, MM, MMR, 104, "myanmar"],
    [153, NA, NAM, 516, "namibia"],
    [154, NR, NRU, 520, "nauru"],
    [155, NP, NPL, 524, "nepal"],
    [156, NL, NLD, 528, "netherlands", "holland"],
    [157, AN, ANT, 530, "netherlands antilles"],
    [158, NC, NCL, 540, "new caledonia"],
    [159, NZ, NZL, 554, "new zealand"],
    [160, NI, NIC, 558, "nicaragua"],
    [161, NE, NER, 562, "niger"],
    [162, NG, NGA, 566, "nigeria"],
    [163, NU, NIU, 570, "niue"],
    [164, NF, NFK, 574, "norfolk", "norfolk island"],
    [165, MP, MNP, 580, "mariana", "mariana islands", "northern mariana islands"],
    [166, NO, NOR, 578, "norway"],
    [167, OM, OMN, 512, "oman"],
    [168, PK, PAK, 586, "pakistan"],
    [169, PW, PLW, 585, "palau"],
    [170, PS, PSE, 275, "palestine", "palestinian territory"],
    [171, PA, PAN, 591, "panama"],
    [172, PG, PNG, 598, "papua", "new guinea", "papua new guinea"],
    [173, PY, PRY, 600, "paraguay"],
    [174, PE, PER, 604, "peru"],
    [175, PH, PHL, 608, "philippines"],
    [176, PN, PCN, 612, "pitcairn"],
    [177, PL, POL, 616, "poland"],
    [178, PT, PRT, 620, "portugal"],
    [179, PR, PRI, 630, "puerto", "rico", "puerto rico"],
    [180, QA, QAT, 634, "qatar"],
    [181, RE, REU, 638, "reunion", "réunion"],
    [182, RO, ROU, 642, "romania"],
    [183, RU, RUS, 643, "russia", "russian", "russian federation"],
    [184, RW, RWA, 646, "rwanda"],
    [185, BL, BLM, 652, "barthelemy", "barthélemy", "saint-barthélemy", "saint-barthelemy"],
    [186, SH, SHN, 654, "helena", "saint helena"],
    [187, KN, KNA, 659, "kitts", "vevis", "saint kitts and vevis"],
    [188, LC, LCA, 662, "lucia", "saint lucia"],
    [189, MF, MAF, 663, "martin", "saint-martin", "saint-martin (french part)"],
    [190, PM, SPM, 666, "saint pierre", "pierre", "miquelon", "saint pierre and miquelon"],
    [191, VC, VCT, 670, "saint vincent", "vincent", "grenadines", "saint vincent and grenadines"],
    [192, WS, WSM, 882, "samoa"],
    [193, SM, SMR, 674, "marino", "san marino"],
    [194, ST, STP, 678, "tome", "sao tome", "principe", "sao tome and principe"],
    [195, SA, SAU, 682, "saudi", "saudi arabia"],
    [196, SN, SEN, 686, "senegal"],
    [197, RS, SRB, 688, "serbia"],
    [198, SC, SYC, 690, "seychelles"],
    [199, SL, SLE, 694, "sierra", "leone", "sierra leone"],
    [200, SG, SGP, 702, "singapore"],
    [201, SK, SVK, 703, "slovakia"],
    [202, SI, SVN, 705, "slovenia"],
    [203, SB, SLB, 090, "solomon", "solomon islands"],
    [204, SO, SOM, 706, "somalia"],
    [205, ZA, ZAF, 710, "south africa"],
    [206, GS, SGS, 239, "south georgia", "south sandwich islands", "south georgia and the south sandwich islands"],
    [207, SS, SSD, 728, "south sudan"],
    [208, ES, ESP, 724, "spain"],
    [209, LK, LKA, 144, "sri", "lanka", "sri lanka"],
    [210, SD, SDN, 736, "sudan"],
    [211, SR, SUR, 740, "suriname"],
    [212, SJ, SJM, 744, "svalbard", "jan mayen", "jan mayen islands", "svalbard and jan mayen islands"],
    [213, SZ, SWZ, 748, "swaziland"],
    [214, SE, SWE, 752, "sweden"],
    [215, CH, CHE, 756, "switzerland"],
    [216, SY, SYR, 760, "syria", "syrian arab republic", "syrian arab republic (syria)"],
    [217, TW, TWN, 158, "taiwan", "taiwan, republic of china"],
    [218, TJ, TJK, 762, "tajikistan"],
    [219, TZ, TZA, 834, "tanzania", "tanzania, united republic of"],
    [220, TH, THA, 764, "thailand"],
    [221, TL, TLS, 626, "timor", "leste", "timor-leste"],
    [222, TG, TGO, 768, "togo"],
    [223, TK, TKL, 772, "tokelau"],
    [224, TO, TON, 776, "tonga"],
    [225, TT, TTO, 780, "trinidad", "tobago", "trinidad and tobago"],
    [226, TN, TUN, 788, "tunisia"],
    [227, TR, TUR, 792, "turkey"],
    [228, TM, TKM, 795, "turkmenistan"],
    [229, TC, TCA, 796, "turks", "caicos", "turks and caicos islands"],
    [230, TV, TUV, 798, "tuvalu"],
    [231, UG, UGA, 800, "uganda"],
    [232, UA, UKR, 804, "ukraine"],
    [233, AE, ARE, 784, "emirates", "united arab emirates"],
    [234, GB, GBR, 826, "great britain", "england", "scotland", "united kingdom"],
    [235, US, USA, 840, "united states", "america", "united states of america"],
    [236, UM, UMI, 581, "minor outlying islands", "us minor outlying islands"],
    [237, UY, URY, 858, "uruguay"],
    [238, UZ, UZB, 860, "uzbekistan"],
    [239, VU, VUT, 548, "vanuatu"],
    [240, VE, VEN, 862, "venezuela", "bolivarian republic", "venezuela (bolivarian republic)"],
    [241, VN, VNM, 704, "vietnam", "viet nam"],
    [242, VI, VIR, 850, "virgin islands, us"],
    [243, WF, WLF, 876, "wallis", "futuna", "futuna islands", "wallis and futuna islands"],
    [244, EH, ESH, 732, "western sahara"],
    [245, YE, YEM, 887, "yemen"],
    [246, ZM, ZMB, 894, "zambia"],
    [247, ZW, ZWE, 716, "zimbabwe"],
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
