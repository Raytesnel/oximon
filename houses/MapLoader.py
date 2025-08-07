import arcade
from arcade import TileMap, View
from loguru import logger

from lifeforms.characters import Player

class HouseMap(arcade.View):
    def __init__(self, player:Player, overworld_map:View,tile_map:TileMap):
        super().__init__()
        self.world = map
        self.player_list = arcade.SpriteList()
        self.player = player
        self.player_list.append(self.player )
        self.all_sprites = []
        self.tile_map = tile_map
        self.scene=None
        self.camera = arcade.Camera2D()



    def setup(self):
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.walls = self.scene["abandoned"]
        try:
            start = next((o for o in self.tile_map.object_lists["objects"] if o.name == "player-start"), None)
            logger.debug("found player start")
        except KeyError:
            raise ValueError(" player start not found")

        self.player.center_x = start.shape[0][0]
        self.player.center_y = start.shape[0][1]

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.scene.get_sprite_list("voorgrond").draw()
