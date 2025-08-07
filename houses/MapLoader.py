import arcade
from arcade import TileMap, View
from loguru import logger

from lifeforms.characters import Player

class HouseMap(arcade.View):
    def __init__(self, player:Player, overworld_map:View,tile_map:TileMap):
        super().__init__()
        self.world = overworld_map
        self.player_list = arcade.SpriteList()
        self.player = player
        self.player_list.append(self.player )
        self.all_sprites = []
        self.tile_map = tile_map
        self.scene=None
        self.camera = arcade.Camera2D()
        self.setup()

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
        self.scene["achtergrond"].draw()
        self.walls.draw()
        self.player_list.draw()
        self.scene["voorgrond"].draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.scene.update_animation(delta_time)
        self.player_list.update_animation(delta_time)
        if arcade.check_for_collision_with_list(self.player, self.walls):
            self.player.center_x -= self.player.change_x
            self.player.center_y -= self.player.change_y
        self.camera.position = arcade.Vec2(self.player.center_x, self.player.center_y)

    def on_key_press(self, key, modifiers):
        self.player.change_x = 0
        self.player.change_y = 0
        if key == arcade.key.UP:
            self.player.change_y = 3
        elif key == arcade.key.DOWN:
            self.player.change_y = -3
        elif key == arcade.key.LEFT:
            self.player.change_x = -3
        elif key == arcade.key.RIGHT:
            self.player.change_x = 3

    def on_key_release(self, key, modifiers):
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0