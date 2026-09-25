use super::{ChannelRegistry, Tank};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Link {
    tank: String,
    channel: String,
}

impl Link {
    pub fn new(tank: &str, channel: &str) -> Option<Self> {
        let tank = tank.trim();
        if tank.is_empty() {
            return None;
        }
        Some(Self {
            tank: tank.to_ascii_lowercase(),
            channel: ChannelRegistry::normalize(channel)?.into_owned(),
        })
    }

    pub fn channel(&self) -> &str {
        &self.channel
    }

    pub fn reaches(&self, tank: &Tank) -> bool {
        tank.name.eq_ignore_ascii_case(&self.tank)
    }

    pub fn level_in(&self, tanks: &[Tank]) -> bool {
        tanks
            .iter()
            .find(|tank| self.reaches(tank))
            .is_some_and(|tank| tank.channels.level(&self.channel))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Transmission {
    pub link: Link,
    pub level: bool,
}

impl Transmission {
    pub fn land(&self, tanks: &mut [Tank]) {
        let Some(tank) = tanks.iter_mut().find(|tank| self.link.reaches(tank)) else {
            return;
        };
        tank.channels.drive(&self.link.channel, self.level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tank::TankKind;

    fn tank(name: &str) -> Tank {
        Tank::new(name.to_string(), TankKind::Base, &[])
    }

    #[test]
    fn a_link_needs_both_a_tank_and_a_channel() {
        assert_eq!(Link::new("", "y"), None);
        assert_eq!(Link::new("   ", "y"), None);
        assert_eq!(Link::new("Zion", ""), None);
        assert_eq!(Link::new("Zion", "   "), None);
        assert!(Link::new("Zion", "y").is_some());
    }

    #[test]
    fn a_link_is_one_link_however_its_halves_were_typed() {
        let link = Link::new("  Zion ", "  Carry  In ").expect("both halves are there");
        assert_eq!(link.channel(), "carry in");
        assert_eq!(Some(link), Link::new("ZION", "carry in"));
    }

    #[test]
    fn a_link_reaches_its_tank_however_the_name_was_typed() {
        let link = Link::new("zion", "y").expect("a link");
        assert!(link.reaches(&tank("Zion")));
        assert!(!link.reaches(&tank("Fishtank")));
    }

    #[test]
    fn a_link_reads_the_far_channel_and_nothing_when_the_tank_is_gone() {
        let mut tanks = vec![tank("Fishtank"), tank("Zion")];
        tanks[1].channels.set_level("y", true);
        assert!(Link::new("Zion", "y").expect("a link").level_in(&tanks));
        assert!(!Link::new("Fishtank", "y").expect("a link").level_in(&tanks));
        assert!(!Link::new("Atlantis", "y").expect("a link").level_in(&tanks));
    }

    #[test]
    fn a_transmission_lands_on_its_tank_alone_and_only_when_the_stage_commits() {
        let mut tanks = vec![tank("Fishtank"), tank("Zion")];
        let sent = Transmission {
            link: Link::new("Zion", "y").expect("a link"),
            level: true,
        };
        sent.land(&mut tanks);
        assert!(
            !tanks[1].channels.level("y"),
            "a drive waits for the commit"
        );
        tanks[1].channels.commit();
        assert!(tanks[1].channels.level("y"));
        assert!(tanks[0].channels.is_empty(), "the home tank hears nothing");
    }

    #[test]
    fn a_transmission_to_a_tank_that_is_gone_lands_nowhere() {
        let mut tanks = vec![tank("Fishtank")];
        Transmission {
            link: Link::new("Atlantis", "y").expect("a link"),
            level: true,
        }
        .land(&mut tanks);
        tanks[0].channels.commit();
        assert!(tanks[0].channels.is_empty());
    }
}
