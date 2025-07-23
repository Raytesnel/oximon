import arcade
import os

SCREEN_WIDTH = 800
SCREEN_HEIGHT = 600
SCREEN_TITLE = "PokeSmash"

ASSETS_PATH = os.path.join(os.path.dirname(__file__), "assets")


class Player(arcade.Sprite):
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

        # Update direction based on velocity
        if abs(self.change_x) > abs(self.change_y):
            self.direction = "right" if self.change_x > 0 else "left"
        else:
            self.direction = "up" if self.change_y > 0 else "down"

        # Animate over time
        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.animations[self.direction])
            self.texture = self.animations[self.direction][self.current_frame]
            self.frame_timer = 0


class WalkDemo(arcade.Window):
    def __init__(self):
        super().__init__(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
        arcade.set_background_color(arcade.color.SKY_BLUE)

        self.player = None
        self.player_list = None
        self.camera = None
        self.scene = None
        self.map_sprite=None

    def setup(self):
        self.player_list = arcade.SpriteList()
        asset_path = os.path.join(ASSETS_PATH, "sprites/player")
        self.camera = arcade.Camera2D()
        self.scene = arcade.Scene()
        self.player = Player(asset_path)
        self.player.center_x = SCREEN_WIDTH // 2
        self.player.center_y = SCREEN_HEIGHT // 2
        self.player.scale = 2.0
        self.player.center_x = -980  # X position in pixels
        self.player.center_y = 970  # Y position in pixels
        self.player_list.append(self.player)
        self.map_sprite = arcade.Sprite(os.path.join(ASSETS_PATH, "map/platform.png"))
        self.map_sprite.center_x = self.map_sprite.width // 2
        self.map_sprite.center_y = self.map_sprite.height // 2
        self.map_sprite.scale = 2.0
        self.scene.add_sprite("Map", self.map_sprite)

    def on_draw(self):
        self.clear()
        self.camera.use()
        self.scene.draw()
        self.player_list.draw()

    def on_update(self, delta_time):
        self.player_list.update()
        self.player_list.update_animation(delta_time)
        self.center_camera_to_player()

    def on_key_press(self, key, modifiers):
        # Cancel movement in both directions first
        self.player.change_x = 0
        self.player.change_y = 0

        # Only one direction active at a time
        if key == arcade.key.UP:
            self.player.change_y = 5
        elif key == arcade.key.DOWN:
            self.player.change_y = -5
        elif key == arcade.key.LEFT:
            self.player.change_x = -5
        elif key == arcade.key.RIGHT:
            self.player.change_x = 5

    def on_key_release(self, key, modifiers):
        # Stop player movement on key release
        if key == arcade.key.UP and self.player.change_y > 0:
            self.player.change_y = 0
        elif key == arcade.key.DOWN and self.player.change_y < 0:
            self.player.change_y = 0
        elif key == arcade.key.LEFT and self.player.change_x < 0:
            self.player.change_x = 0
        elif key == arcade.key.RIGHT and self.player.change_x > 0:
            self.player.change_x = 0

    def center_camera_to_player(self):
        target = arcade.Vec2(
            self.player.center_x,
            self.player.center_y
        )
        self.camera.position = target


if __name__ == "__main__":
    window = WalkDemo()
    window.setup()
    arcade.run()
