use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Instant;

type Graph = Vec<[usize; 2]>;

fn parse_map(input: &str) -> (Graph, HashMap<&str, usize>) {
    let mut split = input.split("\n\n");
    let graph_s = split.nth(1).unwrap();
    let nodes_s: Vec<_> = graph_s
        .lines()
        .map(|s| {
            let node = &s[0..=2];
            let l = &s[7..=9];
            let r = &s[12..=14];
            (node, l, r)
        })
        .collect();
    let node_to_id = nodes_s
        .iter()
        .enumerate()
        .map(|n| (n.1 .0, n.0))
        .collect::<HashMap<&str, usize>>();
    let mut result: Vec<[usize; 2]> = vec![[0, 0]; node_to_id.len()];
    nodes_s.iter().for_each(|(n, l, r)| {
        let n_id = node_to_id.get(n).unwrap();
        let l_id = node_to_id.get(l).unwrap();
        let r_id = node_to_id.get(r).unwrap();
        result[*n_id][0] = *l_id;
        result[*n_id][1] = *r_id;
    });

    (result, node_to_id)
}

fn parse_path(input: &str) -> Vec<usize> {
    let mut split = input.split("\n\n");
    let path_s = split.next().unwrap();
    path_s
        .chars()
        .map(|c| match c {
            'L' => 0,
            'R' => 1,
            _ => panic!("Invalid char {c}"),
        })
        .collect()
}

fn count_nodes_til(graph: &Graph, path: &Vec<usize>, start_id: usize, end_id: usize) -> usize {
    let mut n = start_id;
    let mut ct = 0;
    let mut path_idx = 0;
    while n != end_id {
        ct += 1;
        let dir = path[path_idx];
        n = graph[n][dir];
        path_idx += 1;
        if path_idx >= path.len() {
            path_idx = 0;
        }
    }
    ct
}

pub fn step(graph: &Graph, id: usize, dir: usize) -> usize {
    graph[id][dir]
}

//fn solve_pt2_mulithread(
//    graph: &Graph,
//    path: &Vec<usize>,
//    node_to_id: &HashMap<&str, usize>,
//) -> usize {
//    let z_nodes: HashSet<usize> = node_to_id
//        .iter()
//        .filter_map(|(s, i)| match s.chars().last().unwrap() {
//            'Z' => Some(*i),
//            _ => None,
//        })
//        .collect();
//
//    let n_threads = 10;
//    let mut handles = Vec::with_capacity(n_threads);
//
//    let barrier = Arc::new(Barrier::new(n_threads));
//    let
//    for _ in 0..n_threads {
//        let c = Arc::clone(&barrier);
//    }
//
//    todo!()
//}

pub fn find_loop(graph: &Graph, path: &Vec<usize>, start_id: usize) -> (Vec<usize>, usize) {
    let mut visited = HashMap::<(usize, usize), usize>::default();
    let mut cur_id = start_id;
    let mut history = Vec::<usize>::default();
    let mut path_idx = 0;
    while !visited.contains_key(&(cur_id, path_idx)) {
        visited.insert((cur_id, path_idx), history.len());
        history.push(cur_id);
        let dir = path[path_idx];
        cur_id = graph[cur_id][dir];
        path_idx = if path_idx + 1 >= path.len() {
            0
        } else {
            path_idx + 1
        }
    }
    let loop_start = visited.get(&(cur_id, path_idx)).unwrap();
    let loop_length = visited.len() - loop_start;
    println!("{loop_start}, {loop_length}");
    (history, *loop_start)
}

type Loops = Vec<(Vec<usize>, usize)>;

struct SharedData {
    buffer_barrier: Barrier,
    check_barrier: Barrier,
    loops: Loops,
    found_step: AtomicUsize,
    counter: AtomicUsize,
    buffer: Vec<usize>,
}

impl SharedData {
    fn new(n_threads: usize, buffer_size: usize, loops: &Loops) -> Self {
        SharedData {
            buffer_barrier: Barrier::new(n_threads),
            check_barrier: Barrier::new(n_threads),
            loops: loops.clone(),
            found_step: AtomicUsize::new(0),
            counter: AtomicUsize::new(0),
            buffer: vec![0; buffer_size],
        }
    }
}

fn process_buffer(
    n_threads: usize,
    thread_id: usize,
    path_idx: &mut usize,
    shared_data: &SharedData,
) {
    let process_len = shared_data.buffer.len() / n_threads;
    let start_idx = process_len * thread_id;

    let mut data_ptr = unsafe { (shared_data.buffer.as_ptr() as *mut usize).add(start_idx) };
    let the_loop = &shared_data.loops[thread_id].0;
    let loop_start = shared_data.loops[thread_id].1;
    for _ in 0..process_len {
        unsafe { *data_ptr = the_loop[*path_idx] }
        data_ptr = unsafe { data_ptr.add(1) };
        *path_idx += 1;
        if *path_idx >= the_loop.len() {
            *path_idx = loop_start;
        }
    }
}

fn check_matching_node(n_threads: usize, thread_id: usize, shared_data: &SharedData) {}

fn run_thread(n_threads: usize, thread_id: usize, shared_data: &SharedData) {
    // Using found_step like this does not allow the possiblity for the found node to be
    // on the first node. If this were real, need to add a pre-check if the first node is
    // match

    loop {
        let mut step_count = 0;
        let mut path_idx = 0;
        let found = unsafe { *shared_data.found_step.as_ptr() } != 0;
        if !found {
            // Process the buffer
            process_buffer(n_threads, thread_id, &mut path_idx, shared_data);
            shared_data.buffer_barrier.wait();
            // Check for matching node in the buffer
            shared_data.check_barrier.wait();
        }
    }
}

fn find_count_multithreaded(loops: &Vec<(Vec<usize>, usize)>) -> usize {
    let n_threads = loops.len(); // or number of cores
    let buffer_size = 100000 * loops.len();

    let shared_data = Arc::new(SharedData::new(n_threads, buffer_size, loops));

    let mut handles = Vec::with_capacity(n_threads);
    for thread_id in 0..n_threads {
        let data = shared_data.clone();
        handles.push(thread::spawn(move || {
            run_thread(n_threads, thread_id, &data);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    todo!()
}

fn find_count(loops: &Vec<(Vec<usize>, usize)>) -> usize {
    let mut found = false;
    let mut steps = vec![0; loops.len()];
    let mut ct = 0;
    let now = Instant::now();
    while !found {
        found = loops.iter().enumerate().all(|(i, l)| {
            let is_node = l.0[steps[i]] == 1;
            steps[i] = if steps[i] + 1 >= l.0.len() {
                l.1
            } else {
                steps[i] + 1
            };
            is_node
        });
        ct += 1;

        if (ct % 10000000 == 0) {
            let elapsed = now.elapsed().as_millis();
            println!("{}/s", ct as u128 / elapsed)
        }
    }
    ct
}

pub fn solve_pt2_faster(
    graph: &Graph,
    path: &Vec<usize>,
    node_to_id: &HashMap<&str, usize>,
) -> usize {
    let z_nodes: Vec<usize> = {
        let z_nodes: HashSet<usize> = node_to_id
            .iter()
            .filter_map(|(s, i)| match s.chars().last().unwrap() {
                'Z' => Some(*i),
                _ => None,
            })
            .collect();
        let mut vec = vec![0; node_to_id.len()];
        z_nodes.iter().for_each(|x| vec[*x] = 1);
        vec
    };

    let a_nodes: HashSet<usize> = node_to_id
        .iter()
        .filter_map(|(s, i)| match s.chars().last().unwrap() {
            'A' => Some(*i),
            _ => None,
        })
        .collect();

    let loops: Vec<_> = a_nodes
        .iter()
        .map(|x| {
            let (mut nodes, loop_start) = find_loop(graph, path, *x);
            nodes
                .iter_mut()
                .for_each(|x| *x = if z_nodes.contains(x) { 1 } else { 0 });
            (nodes, loop_start)
        })
        .collect();
    find_count(&loops);

    todo!()
}

// ~7200/s
pub fn solve_pt2(graph: &Graph, path: &Vec<usize>, node_to_id: &HashMap<&str, usize>) -> usize {
    let mut nodes: Vec<usize> = node_to_id
        .iter()
        .filter_map(|(s, i)| match s.chars().last().unwrap() {
            'A' => Some(*i),
            _ => None,
        })
        .collect();
    let z_nodes: Vec<usize> = {
        let z_nodes: HashSet<usize> = node_to_id
            .iter()
            .filter_map(|(s, i)| match s.chars().last().unwrap() {
                'Z' => Some(*i),
                _ => None,
            })
            .collect();
        let mut vec = vec![0; node_to_id.len()];
        z_nodes.iter().for_each(|x| vec[*x] = 1);
        vec
    };

    let now = Instant::now();

    let mut found = false;
    let mut ct = 0;
    let mut path_idx = 0;
    while !found {
        ct += 1;
        let dir = path[path_idx];
        nodes.iter_mut().for_each(|x| *x = graph[*x][dir]);
        found = nodes.iter().all(|x| z_nodes[*x] == 1);

        path_idx += 1;
        if path_idx >= path.len() {
            path_idx = 0;
        }

        if (ct % 10000000 == 0) {
            let elapsed = now.elapsed().as_millis();
            println!("{}/s", ct as u128 / elapsed)
        }
    }
    ct
}

pub fn solve() {
    let path = parse_path(INPUT);
    let (graph, node_to_id) = parse_map(INPUT);
    //let zzz_id = node_to_id.get(&"ZZZ").unwrap();
    //let aaa_id = node_to_id.get(&"AAA").unwrap();
    //let pt1 = count_nodes_til(&graph, &path, *aaa_id, *zzz_id);
    let pt2 = solve_pt2_faster(&graph, &path, &node_to_id);
    //let pt2 = solve_pt2(&graph, &path, &node_to_id);
    println!("Day 8");
    //println!("Part 1: {pt1}");
    println!("Part 2: {pt2}");
}

static EXAMPLE: &str = "LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)";

static EXAMPLE2: &str = "LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";

static INPUT: &str = "LRRLRLRRLRRRLRLRLRRLRRRLRRRLRRLRRRLRLRLRLRLRLRLRRRLRRLRRRLLLLRRRLRLLLRRRLLRLLRRRLRRRLRLRRLRRRLRRRLLRRRLRLRRRLLRRRLRLLRRRLRRLLRLRLRLRRRLRLLRLRLRRRLRLLRLRLRRRLLRRRLRRLRRRLRLRRLRLRRLRLRRLRRRLLRRRLLLRRRLLRRLRRLRRLRLLRRLRRRLRRLRLRLRRLRRLLLRRLRLRRRLRRRLRRRLLLRLRRRLLRRRLRLLRRRR

NFK = (LMH, RSS)
SLJ = (NBT, CDG)
SKX = (SRC, KKX)
LRB = (MNG, FSH)
QMQ = (DMG, QRV)
VRB = (PGG, PRG)
QNB = (PFD, FFQ)
BPK = (GFF, DFF)
JTD = (NVV, VLD)
PXD = (VNV, BDS)
PXP = (VFH, DLR)
XRH = (QMM, LDH)
GLV = (SDM, MFS)
TTM = (VGH, SJG)
RHC = (DBR, QSQ)
KNP = (FFB, SSB)
VLL = (VNM, XQQ)
SMJ = (PNH, MML)
XJN = (SCD, TPG)
GJL = (RJK, GGH)
NFD = (CKB, HXV)
XPH = (CDT, CVK)
GNJ = (JFV, KLQ)
CKC = (PCP, RXV)
PTN = (MFX, GXV)
HSD = (MGC, CGB)
MFX = (MCK, PKT)
QVQ = (LDG, JNX)
PBB = (DXK, KJL)
VNG = (JCH, GPT)
LFM = (HCK, GGQ)
SGL = (QPC, FJF)
RFK = (VMS, CRB)
TTJ = (RMQ, XHJ)
FJP = (GCL, KXF)
BRS = (LRK, NFD)
HTL = (CBJ, BFM)
QFP = (LRG, XQF)
KKH = (QSQ, DBR)
SLA = (HLN, TMV)
JGS = (VMH, CHF)
VNS = (HLN, TMV)
FFM = (KKX, SRC)
JHJ = (FMR, LVD)
QSQ = (JVR, VVX)
QKN = (LFQ, MDJ)
JDC = (XFF, GFS)
RPC = (TFS, TQL)
HGD = (BHN, HKH)
NDB = (CMJ, KDK)
CQP = (TCK, RFK)
SSB = (SNS, XPX)
VXB = (QRS, SHF)
RRT = (PJX, LTG)
BBQ = (SBX, CFK)
DJD = (KMQ, CHQ)
LXR = (VXS, FLP)
VHD = (QVQ, LQQ)
BFM = (VQS, VMX)
QGL = (CBD, VHD)
MML = (SLT, MCH)
VGV = (SNX, QBN)
KND = (SQS, JFC)
NFX = (XVF, GLX)
HCK = (DFX, JGS)
RSS = (MGG, CNL)
KRH = (RKF, KJR)
GMV = (BLX, FKF)
CFB = (XPC, RRF)
KLQ = (JGV, SNL)
KCR = (NTC, GQL)
SCB = (JSG, VBB)
TSJ = (VSJ, HQD)
SNL = (NNP, XBN)
RXV = (PMB, RRT)
MTH = (PCC, BPK)
RRD = (GNL, MSJ)
NVT = (CMD, FCF)
KNF = (SQQ, JPS)
SMK = (CKC, QTJ)
RFB = (FDN, CJB)
QXD = (NRN, GLV)
LCR = (TRT, NMR)
DPR = (GRM, HLQ)
CMD = (FMV, NGL)
LHP = (VVP, LCR)
NDN = (BRH, PTF)
JFC = (HFV, DLQ)
VMX = (NVL, KVQ)
HJF = (QFD, NDN)
XHC = (JVC, JKS)
LBN = (CCG, JRV)
LPK = (XCM, VXL)
HXJ = (TTM, PPT)
GNL = (HNK, QQJ)
VRK = (MRC, XPH)
DJF = (CLD, NGD)
KVP = (DJD, FXM)
JHQ = (VNS, RPZ)
FSK = (XCM, VXL)
HCJ = (PJB, PBF)
FJC = (TTJ, XHB)
JVR = (VRH, RBM)
NVL = (CQD, PJL)
QNL = (GXD, DHK)
NHK = (RFB, MXR)
VQP = (BBG, PXD)
SPV = (LMN, DTT)
KJR = (PFJ, HPH)
VGH = (PGC, NGH)
SQQ = (DQV, BLM)
DTQ = (SMB, CNC)
RVB = (JRQ, JRQ)
KKF = (SGL, NNN)
MTP = (MHN, JGN)
PNX = (PMS, XHV)
CTR = (XBB, VRB)
DFX = (CHF, VMH)
QQM = (SRG, PSR)
GLQ = (FHJ, TQD)
RLJ = (LNR, NRS)
JLJ = (DRR, MCV)
PRJ = (XHF, SMK)
XKQ = (HCJ, SHG)
HJL = (BBG, PXD)
MGC = (RLV, DPR)
GLX = (RHC, KKH)
AAA = (QRH, FRS)
LSB = (XXB, XKQ)
VNQ = (NVC, XTP)
FJR = (HGD, SBH)
MHX = (LRH, QGL)
LMN = (STQ, STQ)
FPT = (HQH, QQM)
VGB = (MRC, XPH)
VKB = (JXN, SKV)
VHJ = (PRT, DTQ)
QPC = (JLJ, XXR)
CJJ = (KXB, VXB)
SHG = (PBF, PJB)
SKV = (RHQ, VCC)
FMH = (LTT, HNX)
SKQ = (SKX, FFM)
LRG = (VHN, VHN)
GCL = (RBQ, MKB)
XVJ = (PQQ, FNC)
HPV = (XHF, SMK)
MCX = (GDX, JMV)
NNN = (QPC, FJF)
HNK = (CPS, NHK)
VBB = (FDV, FMJ)
TCD = (BFM, CBJ)
TBV = (BSL, FKK)
CPT = (LTT, HNX)
CDL = (DCL, SMS)
JGN = (SHK, KTG)
CRB = (PXP, PPH)
JPS = (DQV, BLM)
MMS = (HJF, MGK)
JFT = (FFQ, PFD)
PGC = (TQP, BDN)
RPZ = (TMV, HLN)
FVD = (GTM, PDN)
RTB = (FTM, QBK)
DML = (XKK, LMP)
QGH = (MHX, PXH)
RHM = (TFS, TQL)
GXB = (LXM, NPC)
CMJ = (LXR, PHP)
FFB = (SNS, XPX)
MKC = (QMM, LDH)
MCK = (TRP, VXJ)
FNL = (QNB, JFT)
GGQ = (JGS, DFX)
NMR = (MQP, TNX)
XKK = (XLK, VKB)
QML = (LBN, FRH)
JCH = (GQH, CLR)
TMV = (BMV, TKL)
FDN = (HCM, VVS)
GPT = (CLR, GQH)
DDC = (QGH, KVN)
DKK = (LBN, FRH)
RBM = (SLJ, NLJ)
KLB = (LFV, SPT)
GRK = (XNP, VLL)
CLD = (CSS, GXT)
GFS = (KRH, TRK)
RQJ = (RQX, SQM)
KTG = (NCX, NGJ)
HNX = (JQB, SBJ)
NTR = (VRB, XBB)
HMF = (BGH, QKN)
TPB = (LMN, DTT)
GHX = (QBK, FTM)
RMT = (RFK, TCK)
MSR = (NMQ, LJC)
PPT = (SJG, VGH)
QMG = (VRG, SFZ)
FKF = (LHJ, QHG)
QRL = (LRD, RHJ)
TFS = (GRK, JTQ)
FMJ = (TSF, SXS)
SBX = (GGT, VCL)
KFG = (FCX, FTF)
VXS = (RBG, KNP)
VDG = (JDT, RPS)
QHD = (RFN, ZZZ)
SFM = (GLL, RQC)
VXL = (DCK, FMG)
CCD = (LMP, XKK)
SJG = (PGC, NGH)
FKK = (BPP, HSD)
TXG = (FDC, JNJ)
RKF = (PFJ, HPH)
PMS = (HMT, XTG)
MHN = (SHK, KTG)
DXK = (FPT, JSJ)
QCQ = (LRM, MCX)
QPR = (MST, QHD)
CKV = (VNS, VNS)
DRJ = (VLD, NVV)
FGT = (RHM, RPC)
LHJ = (BQK, JHJ)
STQ = (CKV, CKV)
HLQ = (FXR, KKJ)
GBQ = (BHT, HJH)
RNH = (KNF, GGX)
VPG = (PRR, FHK)
CTS = (VVP, LCR)
CPS = (MXR, RFB)
DRR = (NFK, TKJ)
MQG = (HJH, BHT)
RTL = (GBQ, MQG)
KSV = (HGD, SBH)
TKJ = (LMH, RSS)
FHF = (VCB, BKJ)
CSL = (JNS, FTP)
NMQ = (XPL, PBB)
CNL = (VJJ, JBV)
QQF = (GRF, DKC)
PJB = (PPD, TBV)
BDN = (HBD, RRD)
VRG = (QDM, DVP)
MGK = (NDN, QFD)
CCG = (BQR, QBM)
MQP = (CLF, HNQ)
TPG = (VPG, BMF)
VPP = (QBN, SNX)
FXM = (KMQ, CHQ)
CDG = (PPP, KFN)
LVA = (QDM, DVP)
QMM = (RFF, LNJ)
LMP = (VKB, XLK)
MHH = (HBN, FJC)
JBC = (LTN, LTN)
MGG = (VJJ, JBV)
MPF = (LXM, NPC)
JGV = (XBN, NNP)
MNK = (VGL, DJB)
FGX = (BVQ, DQK)
XBJ = (RQC, GLL)
KGB = (MCX, LRM)
HXX = (MHH, TXD)
XLK = (SKV, JXN)
FCJ = (VPT, GQC)
SHK = (NCX, NGJ)
QRH = (BTG, FJP)
LXM = (KGB, QCQ)
DXV = (SLM, SCK)
MRC = (CVK, CDT)
QBN = (PKR, KQM)
PPD = (FKK, BSL)
PXH = (QGL, LRH)
XXB = (SHG, HCJ)
DCK = (DSL, DSL)
PMG = (TXD, MHH)
XKG = (TQD, FHJ)
JRV = (QBM, BQR)
VGL = (TGD, XTD)
XCM = (DCK, FMG)
PQN = (RPS, JDT)
JBV = (NTR, CTR)
TXD = (HBN, FJC)
VJJ = (CTR, NTR)
DCL = (SMJ, GXC)
PCP = (RRT, PMB)
LRD = (XHC, PBG)
QTJ = (PCP, RXV)
VMH = (PKK, RTL)
DBP = (PRJ, HPV)
CLR = (VDG, PQN)
FLP = (KNP, RBG)
RJT = (NDB, PTK)
KFN = (CDL, FDM)
DKC = (CJJ, HTB)
XHV = (XTG, HMT)
CFK = (GGT, VCL)
HGX = (RDT, GKM)
CKB = (CQP, RMT)
KVQ = (CQD, PJL)
PSR = (HGX, JLK)
KTC = (JFV, KLQ)
RMN = (SFM, XBJ)
KMQ = (JDC, MRB)
TXB = (FSR, MQD)
BKS = (FLH, QFP)
PFD = (MNK, BKL)
BQR = (LGC, KKF)
NPA = (FCJ, NPF)
FRR = (MMS, TDR)
BSL = (HSD, BPP)
LMH = (MGG, CNL)
TVH = (TBH, DKP)
GXT = (JTD, DRJ)
DFF = (TFC, KNN)
RPS = (DBP, LXB)
HKH = (VNQ, DPG)
NRS = (VGR, DPS)
FKT = (NDB, PTK)
GGX = (SQQ, JPS)
GDA = (LGN, DDC)
RBG = (FFB, SSB)
HBN = (TTJ, XHB)
BLM = (TXG, JVD)
VRH = (NLJ, SLJ)
XPL = (DXK, KJL)
VNM = (XXJ, QNL)
MDC = (MTP, TGC)
GQL = (KSN, VKN)
TNX = (HNQ, CLF)
NVV = (HVF, FGX)
DXM = (PTN, QGM)
XBB = (PRG, PGG)
RLM = (XKQ, XXB)
TGC = (JGN, MHN)
SDM = (SLV, KVS)
BPP = (MGC, CGB)
QVB = (QFM, QMG)
LJC = (PBB, XPL)
QLG = (PPT, TTM)
XTT = (BGH, QKN)
HTB = (VXB, KXB)
VCC = (MKC, XRH)
LRM = (GDX, JMV)
CJB = (HCM, VVS)
TRK = (KJR, RKF)
TFC = (CHT, LBT)
NVC = (TCD, HTL)
JRQ = (LGN, DDC)
SQS = (DLQ, HFV)
CHF = (PKK, RTL)
PGG = (RQJ, PJM)
PRR = (JGP, MSR)
VHN = (QFM, QFM)
RQC = (GNJ, KTC)
MRB = (GFS, XFF)
XBN = (XJN, DXC)
SXS = (BBQ, XFV)
VCB = (LPK, FSK)
FTM = (SPV, TPB)
PJL = (SPJ, VHJ)
BKL = (DJB, VGL)
GRR = (DJD, FXM)
ZZZ = (FRS, QRH)
KDK = (PHP, LXR)
SNX = (KQM, PKR)
VMS = (PXP, PPH)
FSH = (RJT, FKT)
JDT = (DBP, LXB)
KVN = (PXH, MHX)
DHK = (DLT, PNX)
JSG = (FDV, FMJ)
GQT = (NRN, GLV)
NBT = (PPP, KFN)
PMX = (KVT, RMN)
SBH = (HKH, BHN)
MNG = (FKT, RJT)
FNC = (JSV, HKZ)
KXF = (MKB, RBQ)
DTT = (STQ, VFM)
PKT = (VXJ, TRP)
XXR = (DRR, MCV)
VKN = (GRR, KVP)
FJF = (XXR, JLJ)
SMB = (RTB, GHX)
RHJ = (PBG, XHC)
CDR = (RJK, GGH)
RBQ = (PMG, HXX)
XCT = (NTC, GQL)
FMR = (JLQ, QRL)
SNS = (XKG, GLQ)
FSR = (NCR, LRB)
PKK = (GBQ, MQG)
VXJ = (FDS, TSJ)
QTQ = (BLX, FKF)
KSN = (GRR, KVP)
FCX = (RLJ, TQN)
VNV = (VQV, MTH)
HVF = (BVQ, DQK)
CHQ = (JDC, MRB)
MKB = (HXX, PMG)
FRH = (JRV, CCG)
CQC = (QML, DKK)
SPJ = (DTQ, PRT)
HXV = (CQP, RMT)
CBJ = (VMX, VQS)
FHG = (FSR, MQD)
GFF = (KNN, TFC)
XVF = (RHC, KKH)
QRS = (RNL, FGT)
XFV = (CFK, SBX)
HPH = (SXL, CXD)
FHK = (JGP, MSR)
HQH = (SRG, PSR)
LTT = (SBJ, JQB)
GGT = (NFB, GSS)
PRM = (VRR, MDC)
HMT = (CPT, FMH)
DSL = (MST, MST)
TDM = (QML, DKK)
LHX = (FRR, GSC)
LHG = (DKC, GRF)
PNH = (MCH, SLT)
VFH = (HMF, XTT)
VCL = (GSS, NFB)
PPH = (VFH, DLR)
FHJ = (DML, CCD)
LFQ = (LSB, RLM)
RFV = (LRK, NFD)
JSV = (FCJ, NPF)
DVP = (GDC, RNH)
SXL = (VQP, HJL)
NPC = (QCQ, KGB)
JVC = (FGQ, GGJ)
RQX = (TFQ, BCL)
TFQ = (TPT, KLB)
NPL = (DXN, CMZ)
GKM = (MPQ, BTX)
HSS = (KVT, RMN)
JDF = (HXJ, QLG)
XFF = (TRK, KRH)
DBR = (JVR, VVX)
PJM = (SQM, RQX)
MRF = (JFC, SQS)
BQK = (FMR, LVD)
PHP = (VXS, FLP)
LTN = (DXN, DXN)
PPP = (CDL, FDM)
TBH = (SCB, VCK)
CGB = (RLV, DPR)
TDR = (HJF, MGK)
LFV = (PVB, FHF)
NDL = (FLH, QFP)
KHR = (RVB, RVB)
GRM = (FXR, KKJ)
RFF = (FNL, HMG)
JLK = (GKM, RDT)
TQD = (CCD, DML)
FDV = (SXS, TSF)
XTP = (TCD, HTL)
PCC = (GFF, DFF)
LDG = (QQF, LHG)
HLN = (TKL, BMV)
DPG = (XTP, NVC)
MNH = (GLX, XVF)
SCD = (BMF, VPG)
GSS = (MNH, NFX)
NXR = (MRF, KND)
PVB = (VCB, BKJ)
XHF = (QTJ, CKC)
NRN = (MFS, SDM)
RLV = (HLQ, GRM)
DLF = (CMD, FCF)
VQV = (BPK, PCC)
BHT = (GBF, DRL)
HQD = (NVT, DLF)
CHT = (SLH, JGK)
NFF = (VRR, MDC)
KVT = (XBJ, SFM)
DLQ = (LHX, XRD)
NCR = (FSH, MNG)
VDD = (PQQ, PQQ)
JVD = (FDC, JNJ)
VQS = (KVQ, NVL)
NGJ = (XJT, CTJ)
BGT = (RRF, XPC)
QDD = (RVB, BCF)
DPS = (CDR, GJL)
JFV = (JGV, SNL)
SLM = (QTQ, GMV)
SBS = (TDM, CQC)
BHN = (DPG, VNQ)
FLH = (LRG, XQF)
SBJ = (LTB, SKQ)
FXR = (LFM, BGQ)
NGH = (TQP, BDN)
KQM = (SBS, RSQ)
KPB = (SLM, SCK)
HCM = (VFF, DJF)
LXB = (HPV, PRJ)
STZ = (DDC, LGN)
HMG = (QNB, JFT)
FMG = (DSL, QPR)
XPX = (XKG, GLQ)
CBD = (QVQ, LQQ)
LRH = (VHD, CBD)
SMS = (SMJ, GXC)
QQJ = (NHK, CPS)
XQQ = (QNL, XXJ)
LTB = (FFM, SKX)
JLQ = (RHJ, LRD)
TQP = (HBD, RRD)
RFN = (QRH, FRS)
HJH = (GBF, DRL)
RNL = (RHM, RPC)
LVD = (JLQ, QRL)
TQL = (JTQ, GRK)
MPQ = (NFF, PRM)
BKJ = (FSK, LPK)
FTF = (TQN, RLJ)
LDH = (LNJ, RFF)
VFF = (CLD, NGD)
TQN = (LNR, NRS)
GXV = (MCK, PKT)
DXN = (KPB, DXV)
TCK = (CRB, VMS)
HNQ = (VNG, DHL)
JNS = (KHR, KHR)
BBG = (BDS, VNV)
JXN = (RHQ, VCC)
RCA = (KPB, DXV)
BGQ = (HCK, GGQ)
BCF = (JRQ, STZ)
GDC = (GGX, KNF)
DXC = (TPG, SCD)
DLR = (XTT, HMF)
PTK = (CMJ, KDK)
SKP = (TXB, FHG)
PFJ = (CXD, SXL)
KVS = (CFB, BGT)
PRT = (SMB, CNC)
MCV = (TKJ, NFK)
FGQ = (GXB, MPF)
SLH = (FJR, KSV)
MQD = (LRB, NCR)
VRR = (TGC, MTP)
TPT = (SPT, LFV)
DTK = (HXJ, QLG)
CVK = (QSM, TVH)
RRF = (KKN, KFG)
FCF = (FMV, NGL)
DJB = (TGD, XTD)
FTP = (KHR, QDD)
CSS = (DRJ, JTD)
QGM = (MFX, GXV)
CXD = (HJL, VQP)
TRP = (FDS, TSJ)
PQQ = (JSV, JSV)
RHQ = (MKC, XRH)
FFQ = (BKL, MNK)
JGP = (NMQ, LJC)
LGC = (SGL, NNN)
LQQ = (LDG, JNX)
SBV = (JNS, FTP)
XHB = (RMQ, XHJ)
CQD = (VHJ, SPJ)
RDT = (MPQ, BTX)
LHL = (LTN, NPL)
LRK = (CKB, HXV)
MCH = (HSS, PMX)
QFD = (BRH, PTF)
PKR = (RSQ, SBS)
JNX = (LHG, QQF)
KLX = (PTN, QGM)
MDJ = (LSB, RLM)
TGD = (NXR, DGD)
MST = (RFN, RFN)
BCL = (KLB, TPT)
VPT = (VRK, VGB)
JMV = (DTK, JDF)
PJX = (NDL, BKS)
GGJ = (GXB, MPF)
NNP = (XJN, DXC)
PMB = (PJX, LTG)
QFM = (VRG, VRG)
BTX = (PRM, NFF)
DKP = (SCB, VCK)
BMF = (FHK, PRR)
JTQ = (VLL, XNP)
SRG = (JLK, HGX)
RMQ = (RFV, BRS)
JGK = (FJR, KSV)
KNN = (CHT, LBT)
JQB = (SKQ, LTB)
DHL = (GPT, JCH)
NTC = (KSN, VKN)
QRV = (LHP, CTS)
VFM = (CKV, JHQ)
LBT = (SLH, JGK)
GXC = (PNH, MML)
HFV = (LHX, XRD)
TRT = (TNX, MQP)
FDM = (DCL, SMS)
KXB = (SHF, QRS)
VSJ = (NVT, DLF)
KJL = (JSJ, FPT)
LTG = (BKS, NDL)
SQM = (TFQ, BCL)
PTF = (KCP, QMQ)
LGN = (KVN, QGH)
PBG = (JKS, JVC)
DQK = (XFG, XCP)
XFG = (MXD, FVD)
PRG = (RQJ, PJM)
FRS = (BTG, FJP)
XNP = (VNM, XQQ)
XRD = (GSC, FRR)
RSQ = (TDM, CQC)
MFS = (KVS, SLV)
SPT = (PVB, FHF)
DRL = (VPP, VGV)
VVP = (NMR, TRT)
XTG = (CPT, FMH)
SCK = (GMV, QTQ)
VCK = (VBB, JSG)
DQV = (TXG, JVD)
XPC = (KKN, KFG)
KXJ = (FHG, TXB)
MSJ = (QQJ, HNK)
VGR = (GJL, CDR)
QDM = (RNH, GDC)
NGD = (GXT, CSS)
FDS = (HQD, VSJ)
NFB = (NFX, MNH)
CNC = (GHX, RTB)
BDS = (VQV, MTH)
DGD = (KND, MRF)
CTJ = (VDD, XVJ)
JSJ = (QQM, HQH)
GXD = (PNX, DLT)
TSF = (XFV, BBQ)
GLL = (GNJ, KTC)
CMZ = (DXV, KPB)
SRC = (QXD, GQT)
FMV = (JBC, JBC)
NPF = (GQC, VPT)
HBD = (GNL, MSJ)
CDT = (TVH, QSM)
JNJ = (SKP, KXJ)
NGL = (JBC, LHL)
SFZ = (DVP, QDM)
GTM = (DXM, KLX)
GSC = (MMS, TDR)
XJT = (VDD, VDD)
FDC = (SKP, KXJ)
BVQ = (XCP, XFG)
GGH = (CSL, SBV)
BMV = (XCT, KCR)
QSM = (DKP, TBH)
TKL = (KCR, XCT)
CLF = (DHL, VNG)
PBF = (PPD, TBV)
GBF = (VGV, VPP)
NCX = (XJT, CTJ)
DMG = (LHP, CTS)
MXD = (PDN, GTM)
QBM = (KKF, LGC)
SLV = (BGT, CFB)
BRH = (QMQ, KCP)
XHJ = (BRS, RFV)
MXR = (FDN, CJB)
RJK = (CSL, SBV)
GDX = (DTK, JDF)
VVX = (VRH, RBM)
BGH = (MDJ, LFQ)
HKZ = (NPF, FCJ)
NLJ = (CDG, NBT)
BTG = (KXF, GCL)
QBK = (SPV, TPB)
SHF = (FGT, RNL)
XQF = (VHN, QVB)
JKS = (GGJ, FGQ)
GRF = (HTB, CJJ)
LNR = (VGR, DPS)
XTD = (DGD, NXR)
GQC = (VGB, VRK)
KCP = (DMG, QRV)
XCP = (MXD, FVD)
XXJ = (DHK, GXD)
VVS = (DJF, VFF)
QHG = (BQK, JHJ)
SLT = (PMX, HSS)
PDN = (KLX, DXM)
DLT = (XHV, PMS)
KKN = (FTF, FCX)
BLX = (QHG, LHJ)
GQH = (VDG, PQN)
VLD = (FGX, HVF)
KKX = (GQT, QXD)
KKJ = (BGQ, LFM)
LNJ = (FNL, HMG)";
