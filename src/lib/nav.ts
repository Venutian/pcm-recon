export interface NavItem { name: string; icon: string; group: "Scouting" | "Your team" | "League" | "Tools"; }

export const NAV: NavItem[] = [
  { name: "Overview", icon: "overview", group: "Scouting" },
  { name: "Prospects", icon: "prospects", group: "Scouting" },
  { name: "Scout", icon: "scout", group: "Scouting" },
  { name: "Market", icon: "market", group: "Scouting" },
  { name: "Shortlist", icon: "shortlist", group: "Scouting" },
  { name: "Compare", icon: "compare", group: "Scouting" },
  { name: "My team", icon: "team", group: "Your team" },
  { name: "Finances", icon: "finance", group: "Your team" },
  { name: "Kits", icon: "jersey", group: "Your team" },
  { name: "Teams", icon: "teams", group: "League" },
  { name: "Rankings", icon: "rankings", group: "League" },
  { name: "Database", icon: "database", group: "Tools" },
];
