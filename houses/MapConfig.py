import json
from pathlib import Path

import arcade
from loguru import logger

from houses.BaseMap import BaseMap
from houses.MapLoader import HouseMap
from houses.overworld import OverworldView
from utils import SCREEN_HEIGHT, SCREEN_TITLE, SCREEN_WIDTH, ASSETS_PATH, ROOT_DIR


class MapConfigs:
    MAP_REGISTRY = {
        "overworld": OverworldView,
        "house": HouseMap,
        "base": BaseMap,
    }

    def __init__(self):
        with open(ROOT_DIR/"houses"/"map_config.json") as f:
            self.map_connections = json.load(f)

    def load_map_by_id(self, map_id, spawn_point_id):
        logger.debug("going to load a map")
        logger.debug(f"map_id: {map_id}")
        logger.debug(f"spawn_point_id: {spawn_point_id}")
        locations = self.map_connections[map_id][spawn_point_id]
        transfer_to_map_class = locations["map"]
        type_class = self.map_connections[transfer_to_map_class]["_type"]
        map_class = self.MAP_REGISTRY[type_class]
        map_to_travel = map_class(
            player_location_key=locations["spawn"],
            map=ASSETS_PATH / self.map_connections[transfer_to_map_class]["_file"],
            possible_gates=[
                gate
                for gate in self.map_connections[transfer_to_map_class].keys()
                if gate not in ["_type", "_file"]
            ],
        )
        return map_to_travel

    def get_shizzle(self,gate_object:arcade.types.TiledObject)->str:
        logger.debug(f"get_shizzle: {gate_object.name}")
        for key, item in self.map_connections.items():
            for key_map, map_item in item.items():
                if isinstance(map_item, dict):
                    if gate_object.name == key_map:
                        choosen_map:str =  map_item["map"]
                        if choosen_map:
                            return key
        raise ValueError("choosen map couldnt be found json error")
