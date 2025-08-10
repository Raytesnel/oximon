from pathlib import Path

import arcade
from arcade import TileMap, View, load_spritesheet
from loguru import logger

from houses.BaseMap import BaseMap
from houses.npc import NPC
from lifeforms.characters import Player
from utils import ASSETS_PATH


class HouseMap(BaseMap):
    def __init__(self,player_location_key:str,map:Path,possible_gates:list[str]):
        super().__init__(player_location_key=player_location_key,map=map,possible_gates=possible_gates)
        logger.debug(possible_gates)