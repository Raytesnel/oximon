import arcade

from houses.MapConfig import MapConfigs
from utils import SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE

if __name__ == "__main__":
    test = MapConfigs()
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    view = test.load_save()
    window.show_view(view)
    arcade.run()
