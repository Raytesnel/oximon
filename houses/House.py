from pathlib import Path

import arcade
from loguru import logger

from houses.BaseMap import BaseMap
from houses.npc import NPC
from utils import ASSETS_PATH


class HouseMap(BaseMap):

    def __init__(self, map: Path, possible_gates: list[str]):
        self.music = arcade.Sound(
            "/home/raymond/Downloads/The Offspring - Pretty Fly for a White Guy HD.mp3",
            streaming=True,
        )
        self.current_player = self.music.play(0.8)
        super().__init__(
            map=map,
            possible_gates=possible_gates,
            current_player=self.current_player,
        )
        self.npc = NPC(
            sheet_path=ASSETS_PATH / "sprites" / "npcs" / "SpriteSheet.png",
            name="npc_dude",
        )
        self.npc_list.append(self.npc)
        try:
            npc_start = next(
                (
                    o
                    for o in self.tile_map.object_lists["objects"]
                    if o.name == "npc_dude"
                ),
                None,
            )
            logger.debug("found npc start")
        except KeyError:
            raise ValueError(" player npc not found")
        self.npc.position = npc_start.shape

    def on_draw(self):
        super().on_draw()
        self.camera.use()
        self.npc_list.draw()

    def on_update(self, delta_time):
        super().on_update(delta_time)
        if self.dialog and self.dialog.active:
            return
        self.npc_list.update()
        if arcade.check_for_collision_with_list(self.npc, self.walls):
            self.npc.center_x -= self.npc.change_x
            self.npc.center_y -= self.npc.change_y
