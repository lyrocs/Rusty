#!/usr/bin/env python3
"""
Monster Data Converter
Fetches monster data from RagnAPI and converts it to species.json format.
"""

import argparse
import json
import math
import re
import requests


def parse_attack_range(attack_str: str) -> float:
    """Parse attack string like '584 ~ 804 (1)' and return middle value."""
    # Remove commas and extract the two numbers before the parenthesis
    match = re.match(r"([\d,]+)\s*~\s*([\d,]+)", attack_str)
    if match:
        min_atk = int(match.group(1).replace(",", ""))
        max_atk = int(match.group(2).replace(",", ""))
        return (min_atk + max_atk) / 2
    return 0


def parse_defense(def_str: str) -> float:
    """Parse defense string like '5 + 10' and return sum."""
    # Split by '+' and sum all parts
    parts = def_str.split("+")
    total = 0
    for part in parts:
        try:
            total += int(part.strip())
        except ValueError:
            pass
    return total


def parse_number(num_str: str) -> float:
    """Parse number string, removing commas."""
    return float(num_str.replace(",", ""))


def element_from_type(type_str: str, element_power: int) -> str:
    """Map RO element type to game element."""
    element_map = {
        "neutral": "earth",
        "water": "water",
        "fire": "fire",
        "wind": "wind",
        "earth": "earth",
        "poison": "shadow",
        "holy": "holy",
        "shadow": "shadow",
        "ghost": "ghost",
        "undead": "shadow",
    }
    return element_map.get(type_str.lower(), "earth")


def calculate_base_atk(attack_mid: float, aspd: float) -> int:
    """
    Calculate base_atk using formula:
    base_atk = 30 + log₁₀(ATK_mid / APS) × 35
    where APS = 50 / (200 - ASPD)
    """
    aps = 50 / (200 - aspd)
    if attack_mid / aps <= 0:
        return 0
    base_atk = 30 + math.log10(attack_mid / aps) * 35
    return max(0, round(base_atk))


def calculate_base_def(defense: float) -> int:
    """
    Calculate base_def using formula:
    base_def = 25 + log₁₀(DEF) × 30
    """
    if defense <= 0:
        return 0
    base_def = 25 + math.log10(defense) * 30
    return max(0, round(base_def))


def calculate_base_hp(hp: float, level: float) -> int:
    """
    Calculate base_hp using formula:
    base_hp = 50 + log₁₀(HP_RO / Level_RO) × 50
    """
    if hp / level <= 0:
        return 50
    base_hp = 50 + math.log10(hp / level) * 50
    return max(50, round(base_hp))


def fetch_monster_data(monster_id: int) -> dict:
    """Fetch monster data from RagnAPI."""
    url = f"https://ragnapi.com/api/v1/old-times/monsters/{monster_id}"
    response = requests.get(url)
    response.raise_for_status()
    return response.json()


def convert_monster(data: dict) -> dict:
    """Convert API monster data to species.json format."""
    main_stats = data.get("main_stats", {})

    # Parse values from API
    attack_str = main_stats.get("attack", "0 ~ 0")
    attack_mid = parse_attack_range(attack_str)

    aspd_str = main_stats.get("aspd", "100")
    aspd = float(aspd_str)

    def_str = main_stats.get("def", "0")
    defense = parse_defense(def_str)

    hp = parse_number(main_stats.get("hp", "100"))
    level = parse_number(main_stats.get("level", "1"))

    base_exp = parse_number(main_stats.get("base_exp", "0"))

    # Calculate converted stats
    base_atk = calculate_base_atk(attack_mid, aspd)
    base_def = calculate_base_def(defense)
    base_hp = calculate_base_hp(hp, level)

    # Get element from type
    element = element_from_type(data.get("type", "neutral"), data.get("element_power", 1))

    # Generate ID from monster_info
    monster_id = data.get("monster_info", "unknown").lower().replace(" ", "_")
    monster_name = data.get("monster_info", "Unknown").title()

    species = {
        "id": monster_id,
        "name": monster_name,
        "element": element,
        "level": int(level),
        "base_hp": base_hp,
        "base_atk": base_atk,
        "base_def": base_def,
        "base_spd": 25,  # Default value, can be adjusted
        "base_exp": int(base_exp),
        "learnable_skills": [
            {"skill_id": "tackle", "level_required": 1}
        ],
        "zones": ["prontera"],  # Default zone
        "swap_talent": None
    }

    return species


def main():
    parser = argparse.ArgumentParser(description="Convert RO monster data to species.json format")
    parser.add_argument("monster_id", type=int, help="Monster ID to fetch and convert")
    parser.add_argument("--raw", action="store_true", help="Also print raw API data")
    args = parser.parse_args()

    print(f"Fetching monster {args.monster_id}...")

    try:
        raw_data = fetch_monster_data(args.monster_id)
    except requests.exceptions.RequestException as e:
        print(f"Error fetching data: {e}")
        return 1

    if args.raw:
        print("\n=== Raw API Data ===")
        print(json.dumps(raw_data, indent=2))

    # Print intermediate calculation values
    main_stats = raw_data.get("main_stats", {})
    attack_str = main_stats.get("attack", "0 ~ 0")
    attack_mid = parse_attack_range(attack_str)
    aspd = float(main_stats.get("aspd", "100"))
    aps = 50 / (200 - aspd)
    def_str = main_stats.get("def", "0")
    defense = parse_defense(def_str)
    hp = parse_number(main_stats.get("hp", "100"))
    level = parse_number(main_stats.get("level", "1"))

    print("\n=== Calculation Details ===")
    print(f"Attack: {attack_str} -> mid: {attack_mid}")
    print(f"ASPD: {aspd} -> APS: {aps:.3f}")
    print(f"ATK/APS: {attack_mid / aps:.2f}")
    print(f"Defense: {def_str} -> total: {defense}")
    print(f"HP: {hp}, Level: {level} -> HP/Level: {hp/level:.2f}")

    # Convert to species format
    species = convert_monster(raw_data)

    print("\n=== Converted Species Data ===")
    print(json.dumps(species, indent=2))

    return 0


if __name__ == "__main__":
    exit(main())
