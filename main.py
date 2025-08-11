from pathlib import Path

import arcade

from houses.MapConfig import MapConfigs
from utils import SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE



if __name__ == "__main__":
    test = MapConfigs()
    window = arcade.Window(SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_TITLE)
    view = test.load_map_by_id("house_1", "exit_1")
    window.show_view(view)
    arcade.run()
