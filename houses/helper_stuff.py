def check_object_collision(player, obj, padding:float=0):
    if obj is None:
        return False
    (left, top), (right, _), (_, _), (_, bottom) = obj.shape
    left -= padding
    right += padding
    top += padding
    bottom -=padding
    return (
        player.right > left and
        player.left < right and
        player.top > bottom and
        player.bottom < top
    )

