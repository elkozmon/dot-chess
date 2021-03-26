use crate::board::{Board, Event, File, Piece, Side, Square};
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use ink_storage::Vec;
use scale::{Decode, Encode};

const ZOBRIST_KEYS: [u32; 781] = [
    1942003009, 974690339, 4076484053, 1214391555, 1168039459, 3954880283, 1887261166, 2805340334,
    1039171167, 47040222, 126385303, 3891657653, 926154016, 1941842831, 2567947926, 3048722653,
    436937843, 1071069909, 1432607408, 2992120632, 2693551823, 3999648306, 1382917800, 3485586812,
    691143062, 2247128374, 3001267194, 3717451471, 2822514256, 224123184, 1265408935, 2342676329,
    4210599198, 380716068, 1871130916, 90961246, 95588603, 1340812404, 3157421886, 3033879438,
    2941032298, 4281984084, 791069444, 3229902840, 1085056429, 1036299096, 2041737730, 477391279,
    3573485951, 2428063086, 4006380305, 498281196, 2650559380, 698823106, 2526859270, 905028922,
    3828454112, 4062114938, 1731613784, 718598495, 2469044938, 826853482, 3435891183, 1206561059,
    4001785945, 1580667040, 841118202, 3118064198, 3007239732, 2697588702, 3241717207, 2896976768,
    2625446214, 1139534747, 1634309592, 3645057414, 842057323, 220539104, 1682148287, 1826194925,
    1291519953, 579747182, 1953610848, 79336613, 2299208228, 2960719262, 1387255659, 3255974975,
    3611257744, 1759702130, 700015659, 1437676123, 1409757097, 340829619, 1592482521, 1013716944,
    3551652262, 2273679108, 924794576, 2772300873, 2588336972, 909468361, 1620374906, 626756526,
    910008001, 3124734859, 663841270, 2293194911, 538944660, 2611845740, 4099608980, 1711815261,
    4294750720, 4128410476, 2403639072, 2815512103, 2024458660, 940037285, 4023757283, 200240863,
    2831417015, 2264783481, 3505216783, 3380810231, 2134455769, 1878596326, 1126834320, 706940952,
    3421258597, 3249823935, 3496038749, 1458433332, 1381746117, 1391204547, 4129495670, 701555344,
    3678075934, 3919513721, 1018152000, 3358521865, 1098453521, 3766175686, 3788514923, 3704330180,
    1867846352, 3503222807, 3666494920, 1369885308, 3208214799, 823948767, 3646992868, 1111544253,
    404535103, 2475672433, 4067731061, 3471407125, 3523190831, 1541509609, 1446428187, 2270508344,
    3853202269, 2964548409, 3769288709, 1182945927, 328012867, 2128873257, 500329508, 2392918934,
    1110921668, 3626077748, 3034882286, 371430535, 4247885265, 3431420837, 1667051082, 735473848,
    3304269238, 3436092049, 3695503070, 163649931, 2458074100, 1963338182, 2280566306, 3446808973,
    720544228, 111285085, 2323570567, 1853011421, 140037107, 1755786149, 1835930235, 1064017356,
    4129216220, 196425906, 1928098796, 4102997964, 2428756682, 1621273485, 3051087958, 3345660902,
    1835861449, 3611995615, 4122799497, 3997827263, 3350049766, 4049993909, 3223282616, 2007484320,
    935983269, 3789339574, 2957145821, 3055965848, 2806257257, 432880052, 1126984834, 1670507736,
    4055113409, 226181967, 1565145549, 1856069178, 3847211894, 2547431162, 2982482594, 2303595301,
    1871916337, 596740066, 3621647208, 448445975, 1411478801, 3618296476, 644632403, 3955435379,
    1564726818, 1383028689, 1435379756, 1412108725, 3656530592, 2376594121, 179675249, 3213703592,
    4283769767, 2656788170, 699695232, 1510579245, 558399257, 31064803, 3866709247, 3458601848,
    1657272861, 1463510220, 1844499476, 1772313200, 1565639145, 3996115400, 4191519400, 794681510,
    3740433351, 2026179775, 4228218200, 1645888103, 876316018, 1397203474, 298129039, 681329024,
    26210277, 1728471080, 3688381217, 875750524, 4205752235, 2173692201, 3838502371, 3311900884,
    764462908, 127520456, 2342186665, 2240000119, 3910191122, 2529589702, 3880791030, 206354022,
    2438591974, 2584508167, 3583430374, 460440746, 2655957564, 2369033091, 2856340915, 1693076043,
    2321152957, 852808412, 1206783759, 480476928, 3965590587, 3226156208, 21118103, 4199706749,
    2764530386, 2783883919, 1295195544, 2729792514, 385866972, 964768917, 230550668, 4269827214,
    1981581110, 971117672, 1346511909, 2474139887, 3116572519, 570324132, 2228660402, 3113057260,
    1695631712, 1886789841, 1000772865, 48367360, 3879812407, 4259463310, 3577625728, 18613629,
    971517321, 4274174783, 4052895738, 2431456657, 2203553140, 3345169402, 3894997949, 861572100,
    1685690454, 1024368897, 298023905, 2741406378, 3563339263, 1196481911, 3595302333, 3603297915,
    1768583838, 2783181229, 3250302730, 1249806972, 2327832517, 893030764, 264525874, 1569152555,
    3047445695, 3079046245, 2052759526, 3833828976, 2374952697, 4070678605, 238534990, 264298862,
    1196061521, 746356752, 1240977025, 368618913, 3027185624, 426815372, 875091388, 3763023820,
    4107452793, 610950779, 3439405864, 2728680793, 2668021364, 2093030521, 486334561, 4015963872,
    1113087140, 2801426319, 3632791464, 4271441918, 4067385218, 2787746517, 445175918, 232380386,
    4134831302, 2080849244, 1754386390, 112495709, 1259866809, 874189000, 1135076107, 1539691454,
    354561437, 4067652475, 4057813544, 3922535046, 1263184767, 1032412787, 540724737, 1559933735,
    4251627816, 1960531324, 3466654378, 3093348231, 3042857810, 422643440, 424605223, 296744337,
    2557011000, 2008997822, 693146483, 3486437519, 2518837136, 951887810, 1820864461, 939479979,
    2097592300, 957882415, 488296993, 2055870163, 963770112, 1352426313, 3926574426, 2511048387,
    1706998055, 3320297634, 3943062684, 1525901377, 2989334409, 2264393140, 2515718407, 1271756371,
    138143119, 51643617, 1854818891, 4130490374, 923022458, 4168831388, 3200552090, 3096646563,
    3495335162, 2874868322, 1690972388, 3127606884, 124858764, 3581331210, 1996736078, 3038422235,
    620443822, 4195991855, 1673949374, 3208966973, 3666998156, 4034492726, 3907434110, 3964725734,
    38284250, 1667559137, 1587744272, 2418542249, 1773925495, 2966783129, 669351410, 824518505,
    1202601843, 1188804626, 3964967782, 2079913192, 706194718, 2095169054, 3776932048, 1864903972,
    2622648878, 1581857207, 175303527, 3078109944, 4223684060, 1857580227, 676413964, 779269213,
    1803947081, 2841617176, 1849828533, 3447576334, 699836568, 4145216427, 659455824, 2959068012,
    830444492, 3807780294, 1851832815, 2804473686, 1842611746, 74694231, 414438346, 1667062717,
    1220710955, 203314967, 238709240, 2108640105, 2449196068, 4117100008, 116243362, 1668702419,
    2047522721, 4001808889, 1962482506, 3836078388, 3201961, 2898940988, 1580475809, 2074391640,
    3371828045, 4111747256, 2248579464, 2639304252, 3905767865, 1119727231, 481258537, 1369837604,
    360055209, 321548474, 1037035024, 1686381412, 1584788736, 1311465522, 1933321931, 4273966927,
    4009997229, 2754466879, 197642927, 2599710720, 3704078180, 2240175591, 797740404, 513318851,
    1436582145, 3920779415, 1465193651, 3943725068, 2920655887, 3152199518, 1421216623, 3139387502,
    593865014, 1449989416, 3608466360, 3643136674, 1209438496, 304750066, 674473735, 3722475182,
    1266767728, 293597516, 2851296515, 3822349589, 1168496863, 1751283287, 2695490865, 3439111639,
    825937231, 497243397, 2553450708, 1693294832, 1738746412, 1233521988, 187858091, 2322236572,
    244890436, 2522138713, 46244302, 4246313537, 643530656, 3897174335, 1769252821, 316988483,
    3191988266, 194168216, 2716320412, 1355418726, 2632070685, 325865593, 223566820, 296068894,
    4246567234, 2754084406, 1959506124, 1586422739, 615856435, 2816990323, 2964089140, 4126885904,
    3323639088, 416409715, 2737777269, 54049914, 2902455953, 4054498935, 3780588458, 4132643091,
    638327422, 1338562304, 1780101208, 415717705, 2781351661, 1033042505, 163453406, 710278388,
    454211306, 203294494, 709888736, 2639367641, 31088929, 2324274893, 1944169817, 820979112,
    2510135006, 1413189662, 776960456, 1640643882, 1360250075, 596961585, 1815925839, 3932211726,
    2037589618, 2868893771, 2324872756, 2449714202, 821723198, 3881674365, 3576142534, 4100248624,
    4035331610, 1300571397, 1207413101, 23183332, 2278457645, 1101472926, 42702072, 276206558,
    2053390405, 2658579215, 2110923824, 1131164469, 3728511475, 3933968689, 1949432070, 1771558907,
    2361103726, 1171519189, 166691159, 703642330, 2031935954, 1369197138, 891691754, 3657524750,
    3405022555, 780497794, 1247111389, 3471097204, 446001775, 1229294504, 35759868, 1219036607,
    182994319, 800383469, 812345509, 2029659800, 121881032, 3921382503, 4219653514, 872308992,
    1502373345, 2320619898, 890986853, 1374629499, 2796698966, 3669486531, 4076814233, 2407444249,
    2434725747, 2704958396, 3995879091, 3410282841, 830817612, 3151515851, 2114897412, 320389551,
    3896502859, 240959410, 4257993663, 461498033, 2062330353, 2312511768, 210865537, 1574386905,
    3391367871, 1388038878, 1092185153, 3879815746, 695597819, 4108913177, 204479956, 206989775,
    1263888044, 2089850496, 2036571933, 405280886, 2212697960, 486269613, 25331471, 392244806,
    2291724873, 2893488033, 1901461854, 4032830261, 3071651265, 837115266, 564967797, 3400851398,
    1002213739, 1946698567, 200090072, 866130545, 3617993841, 1654281075, 2266582212, 3863600330,
    1436466581, 1495447007, 2678735259, 2217947017, 1287579330, 7811934, 2529679000, 1215689912,
    1112093464, 3273265716, 950604604, 211517989, 2136794007, 480732146, 2360783139, 3391438700,
    3752126210, 62617669, 1399282902, 2923160396, 2232589370, 1286916489, 2734576221, 578492946,
    2521802165, 3363556836, 3069019429, 3560113981, 4022761278, 3321678037, 428095775, 2461882872,
    2481017608, 3690601128, 1561790653, 1902765713, 190711782, 94939157, 47447519, 4113537505,
    3542401824, 3578950769, 4187500080, 564062853, 2883304400, 1436547877, 26954838, 2089032226,
    2802803278, 1430941261, 2170595482, 2247357720, 1553804480, 4126814597, 298693307, 841725043,
    3805793409, 1528232892, 610015325, 182281659, 1045836970,
];

#[derive(PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(feature = "std", derive(Debug, scale_info::TypeInfo, StorageLayout))]
pub struct ZobristHash(u32);

impl ZobristHash {
    const WHITE_TURN_HASH_KEY_INDEX: usize = 780;

    fn get_piece_hash_key_index(side: Side, piece: Piece, square: Square) -> usize {
        let mut index: usize = match side {
            Side::White => 0,
            Side::Black => 384,
        };

        index += 64
            * match piece {
                Piece::Pawn => 0,
                Piece::Knight => 1,
                Piece::Bishop => 2,
                Piece::Rook => 3,
                Piece::Queen => 4,
                Piece::King => 5,
            };

        let square_index: usize = square.index().into();

        index + square_index
    }

    fn get_en_passant_hash_key_index(file: File) -> usize {
        768 + file.index() as usize
    }

    fn get_queen_castling_right_hash_key_index(side: Side) -> usize {
        match side {
            Side::White => 776,
            Side::Black => 777,
        }
    }

    fn get_king_castling_right_hash_key_index(side: Side) -> usize {
        match side {
            Side::White => 778,
            Side::Black => 779,
        }
    }
}

impl ZobristHash {
    pub fn new(board: &Board) -> Self {
        let mut hash = 0;

        for (side, piece, square) in board.get_pieces().iter() {
            let hash_key_index = Self::get_piece_hash_key_index(*side, *piece, *square);
            let hash_key = ZOBRIST_KEYS[hash_key_index];
            hash ^= hash_key;
        }

        let flags = board.get_flags();

        for file in File::VARIANTS.iter() {
            let file = *file;

            if flags.get_en_passant_open(file) {
                let hash_key_index = Self::get_en_passant_hash_key_index(file);
                let hash_key = ZOBRIST_KEYS[hash_key_index];
                hash ^= hash_key;
            }
        }

        for side in Side::VARIANTS.iter() {
            let side = *side;

            if flags.get_queen_side_castling_right(side) {
                let hash_key_index = Self::get_queen_castling_right_hash_key_index(side);
                let hash_key = ZOBRIST_KEYS[hash_key_index];
                hash ^= hash_key;
            }

            if flags.get_king_side_castling_right(side) {
                let hash_key_index = Self::get_king_castling_right_hash_key_index(side);
                let hash_key = ZOBRIST_KEYS[hash_key_index];
                hash ^= hash_key;
            }
        }

        if flags.get_whites_turn() {
            hash ^= ZOBRIST_KEYS[Self::WHITE_TURN_HASH_KEY_INDEX];
        }

        ZobristHash(hash)
    }

    pub fn apply(&self, events: Vec<Event>) -> Self {
        let mut hash = self.0;

        for event in events.into_iter() {
            match event {
                Event::PieceLeftSquare(side, piece, square)
                | Event::PieceEnteredSquare(side, piece, square) => {
                    let hash_key_index = Self::get_piece_hash_key_index(*side, *piece, *square);
                    let hash_key = ZOBRIST_KEYS[hash_key_index];
                    hash ^= hash_key;
                }
                Event::NextTurn(Side::White) => {
                    hash ^= ZOBRIST_KEYS[Self::WHITE_TURN_HASH_KEY_INDEX];
                }
                Event::QueenCastlingRightLost(side) => {
                    let hash_key_index = Self::get_queen_castling_right_hash_key_index(*side);
                    let hash_key = ZOBRIST_KEYS[hash_key_index];
                    hash ^= hash_key;
                }
                Event::KingCastlingRightLost(side) => {
                    let hash_key_index = Self::get_king_castling_right_hash_key_index(*side);
                    let hash_key = ZOBRIST_KEYS[hash_key_index];
                    hash ^= hash_key;
                }
                Event::EnPassantOpened(square) | Event::EnPassantClosed(square) => {
                    let hash_key_index = Self::get_en_passant_hash_key_index(square.file());
                    let hash_key = ZOBRIST_KEYS[hash_key_index];
                    hash ^= hash_key;
                }
                _ => {}
            }
        }

        ZobristHash(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::ZobristHash;
    use crate::board::{File, Piece, Rank, Side, Square};

    #[test]
    fn white_pawn_a1_hash_key_index() {
        let index = ZobristHash::get_piece_hash_key_index(
            Side::White,
            Piece::Pawn,
            Square::new(File::A, Rank::_1),
        );

        assert_eq!(index, 0);
    }

    #[test]
    fn white_king_h8_hash_key_index() {
        let index = ZobristHash::get_piece_hash_key_index(
            Side::White,
            Piece::King,
            Square::new(File::H, Rank::_8),
        );

        assert_eq!(index, 383);
    }

    #[test]
    fn black_pawn_a1_hash_key_index() {
        let index = ZobristHash::get_piece_hash_key_index(
            Side::Black,
            Piece::Pawn,
            Square::new(File::A, Rank::_1),
        );

        assert_eq!(index, 384);
    }

    #[test]
    fn black_king_h8_hash_key_index() {
        let index = ZobristHash::get_piece_hash_key_index(
            Side::Black,
            Piece::King,
            Square::new(File::H, Rank::_8),
        );

        assert_eq!(index, 767);
    }

    #[test]
    fn en_passant_file_a_hash_key_index() {
        let index = ZobristHash::get_en_passant_hash_key_index(File::A);

        assert_eq!(index, 768);
    }

    #[test]
    fn en_passant_file_h_hash_key_index() {
        let index = ZobristHash::get_en_passant_hash_key_index(File::H);

        assert_eq!(index, 775);
    }

    #[test]
    fn white_queen_castling_right_hash_key_index() {
        let index = ZobristHash::get_queen_castling_right_hash_key_index(Side::White);

        assert_eq!(index, 776);
    }

    #[test]
    fn black_queen_castling_right_hash_key_index() {
        let index = ZobristHash::get_queen_castling_right_hash_key_index(Side::Black);

        assert_eq!(index, 777);
    }

    #[test]
    fn white_king_castling_right_hash_key_index() {
        let index = ZobristHash::get_king_castling_right_hash_key_index(Side::White);

        assert_eq!(index, 778);
    }

    #[test]
    fn black_king_castling_right_hash_key_index() {
        let index = ZobristHash::get_king_castling_right_hash_key_index(Side::Black);

        assert_eq!(index, 779);
    }

    #[test]
    fn whites_turn_hash_key_index() {
        assert_eq!(ZobristHash::WHITE_TURN_HASH_KEY_INDEX, 780);
    }
}
