
use text_io;

use std::str::FromStr;
use text_io::try_scan;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Claim {
    id: usize,
    offset: (u32, u32),
    rect: (u32, u32),
}

impl FromStr for Claim {
    type Err = text_io::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut claim = Claim::default();
        // #123 @ 3,2: 5x4
        try_scan!(input.bytes() => "#{} @ {},{}: {}x{}", claim.id, claim.offset.0, claim.offset.1, claim.rect.0, claim.rect.1);
        Ok(claim)
    }
}

const EDGE: usize = 1000;
pub type Field = [[u8; EDGE]; EDGE];

pub fn apply_claims(claims: &[Claim]) -> Field {
    let mut field = [[0_u8; EDGE]; EDGE];

    for claim in claims {
        for x in (claim.offset.0)..(claim.offset.0 + claim.rect.0) {
            let x = x as usize;
            for y in (claim.offset.1)..(claim.offset.1 + claim.rect.1) {
                let y = y as usize;
                field[y][x] = field[y][x].saturating_add(1);
            }
        }
    }

    field
}

pub fn find_uncontended(claims: &[Claim], field: &Field) -> usize {
    // the uncontended claim is the one for which all values in the field are 1

    'claims: for claim in claims {
        for x in (claim.offset.0)..(claim.offset.0 + claim.rect.0) {
            let x = x as usize;
            for y in (claim.offset.1)..(claim.offset.1 + claim.rect.1) {
                let y = y as usize;
                if field[y][x] != 1 {
                    continue 'claims;
                }
            }
        }
        // if we get here, we found the non-overlapped claim
        return claim.id
    }
    0
}
