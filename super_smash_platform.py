import os

import arcade

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")

class SmashStageOnlyView(arcade.View):
    def __init__(self):
        super().__init__()
        self.platforms = arcade.SpriteList()
        self.camera = arcade.Camera2D()
    def setup(self) -> None:
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.player_list = arcade.SpriteList()
        self.character_1 = Character(asset_path)
        self.character_1.center_x = 200
        self.character_1.center_y = 150
        self.character_2 = Character(asset_path)
        self.character_2.center_x = 700
        self.character_2.center_y = 150
        self.player_list.append(self.character_1)
        self.player_list.append(self.character_2)


    def on_show(self):
        ground = arcade.SpriteSolidColor(700, 30, arcade.color.GREEN)
        ground.center_x = self.window.width // 2
        ground.center_y = 150
        self.platforms.append(ground)

    def on_draw(self):
        self.clear()
        self.setup()
        self.on_show()
        self.camera.use()
        self.platforms.draw()
        self.player_list.draw()




    def on_update(self, delta_time):
        self.player_list.update()



class Character(arcade.Sprite):
    def __init__(self, asset_path):
        super().__init__()
        self.animations = {
            "down": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(0, 3)],
            "up": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(3, 6)],
            "left": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(6, 9)],
            "right": [arcade.load_texture(os.path.join(asset_path, f"player_{i}.png")) for i in range(9, 12)],
        }
        self.direction = "down"
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 0.05
        self.texture = self.animations[self.direction][0]

    def update_animation(self, delta_time: float = 1 / 60):
        if self.change_x == 0 and self.change_y == 0:
            self.current_frame = 0
            self.texture = self.animations[self.direction][0]
            return

        if abs(self.change_x) > abs(self.change_y):
            self.direction = "right" if self.change_x > 0 else "left"
        else:
            self.direction = "up" if self.change_y > 0 else "down"

        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.animations[self.direction])
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0

if __name__ == "__main__":
    window = arcade.Window(1280, 720, "Simple Stage")
    window.show_view(SmashStageOnlyView())
    arcade.run()
