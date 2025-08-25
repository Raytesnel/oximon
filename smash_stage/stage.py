import arcade
from arcade import load_tilemap
from loguru import logger


class SmashStage:
    def __init__(self, tmx_path):
        self.tile_map = load_tilemap(tmx_path)
        self.scene = arcade.Scene.from_tilemap(self.tile_map)
        self.platforms = self.scene["platforms"]
        self.background = self.scene["background"]
        self.background_2 = self.scene["background_2"]
        self.death_field = [f for f in self.tile_map.object_lists["death"] if f.name == "Alive"][0]
        self.spawn_points = {obj.name: obj for obj in self.tile_map.object_lists.get("spawn", [])}
        if "player1" not in self.spawn_points or  "player2" not in self.spawn_points:
            raise ValueError("player should be in list")
        self.width = self.tile_map.width * self.tile_map.tile_width
        self.height = self.tile_map.height * self.tile_map.tile_height

    def on_draw(self):
        self.platforms.draw()
        self.background_2.draw()
        self.background.draw()

    def update(self):
        pass

    def check_death_zones(self, player_list)->bool:
        left_top, right_top, right_bottom, left_bottom = self.death_field.shape
        left, right = left_top[0], right_top[0]
        bottom, top = right_bottom[1], right_top[1]

        for character in player_list:
            x, y = character.center_x, character.center_y
            if not (left <= x <= right and bottom <= y <= top):
                character.lives = 0
                logger.debug(f"{character.name} viel uit het speelveld!")
                return True
        return False
