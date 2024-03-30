use crate::uint256::Uint256;

pub struct Hasher {
    p: Uint256,
    n_rounds: u8,
    c: Vec<Uint256>
}

impl Default for Hasher {
    fn default() -> Self {
        Hasher {
            p: Uint256::from("21888242871839275222246405745257275088548364400416034343698204186575808495617"),
            n_rounds: 10,
            c: vec![
                Uint256::from("0"),
                Uint256::from("25823191961023811529686723375255045606187170120624741056268890390838310270028"),
                Uint256::from("71153255768872006974285801937521995907343848376936063113800887806988124358800"),
                Uint256::from("51253176922899201987938365653129780755804051536550826601168630951148399005246"),
                Uint256::from("66651710483985382365580181188706173532487386392003341306307921015066514594406"),
                Uint256::from("45887003413921204775397977044284378920236104620216194900669591190628189327887"),
                Uint256::from("14399999722617037892747232478295923748665564430258345135947757381904956977453"),
                Uint256::from("29376176727758177809204424209125257629638239807319618360680345079470240949145"),
                Uint256::from("13768859312518298840937540532277016512087005174650120937309279832230513110846"),
                Uint256::from("54749662990362840569021981534456448557155682756506853240029023635346061661615"),
                Uint256::from("25161436470718351277017231215227846535148280460947816286575563945185127975034"),
                Uint256::from("90370030464179443930112165274275271350651484239155016554738639197417116558730"),
                Uint256::from("92014788260850167582827910417652439562305280453223492851660096740204889381255"),
                Uint256::from("40376490640073034398204558905403523738912091909516510156577526370637723469243"),
                Uint256::from("903792244391531377123276432892896247924738784402045372115602887103675299839"),
                Uint256::from("112203415202699791888928570309186854585561656615192232544262649073999791317171"),
                Uint256::from("114801681136748880679062548782792743842998635558909635247841799223004802934045"),
                Uint256::from("111440818948676816539978930514468038603327388809824089593328295503672011604028"),
                Uint256::from("64965960071752809090438003157362764845283225351402746675238539375404528707397"),
                Uint256::from("98428510787134995495896453413714864789970336245473413374424598985988309743097")
            ]    
        }
    }
}

impl Hasher {
    fn mimc_feistel(il: &Uint256, ir: &Uint256, k: &Uint256) -> (Uint256, Uint256) {
        let hasher = Hasher::default();
        let mut last_l = il.clone();
        let mut last_r = ir.clone();

        for i in 0..hasher.n_rounds {
            let mask = last_r.add_mod(&k, &hasher.p);
            let mask = mask.add_mod(&hasher.c[i as usize], &hasher.p);
            let mask2 = mask.mul_mod(&mask, &hasher.p);
            let mask4 = mask2.mul_mod(&mask2, &hasher.p);
            let mask = mask4.mul_mod(&mask, &hasher.p);
    
            let temp = last_r;
            last_r = last_l.add_mod(&mask, &hasher.p);
            last_l = temp;
        }
    
        (last_l, last_r)
    }

    pub fn mimc_sponge(left: &Uint256, right: &Uint256, k: &Uint256) -> Uint256 {
        let hasher = Hasher::default();
        let mut last_r = Uint256::new(0);
        let mut last_l = Uint256::new(0);
    
        last_r = last_r.add_mod(&Uint256::new(0), &hasher.p);
        let (new_last_r, new_last_l) = Hasher::mimc_feistel(&last_r, &last_l, &k);
        
        last_r = last_r.add_mod(&Uint256::new(1), &hasher.p);
        let (new_last_r, new_last_l) = Hasher::mimc_feistel(&last_r, &last_l, &k);
    
        last_r
    }
}