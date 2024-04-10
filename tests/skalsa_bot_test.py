import datetime

import pytest

from src.skalsa_bot import get_next_same_weekday


@pytest.mark.parametrize(
    "date, day, expected_result",
    [
        (datetime.datetime(2024, 4, 1), 5, datetime.datetime(2024, 4, 5)),
        (datetime.datetime(2024, 4, 1), 1, datetime.datetime(2024, 4, 2)),
        (datetime.datetime(2024, 4, 1), 3, datetime.datetime(2024, 4, 4)),
        (datetime.datetime(2024, 4, 1), 6, datetime.datetime(2024, 4, 6)),
        (datetime.datetime(2024, 4, 1), 7, datetime.datetime(2024, 4, 7)),
    ],
)
def test_get_next_same_weekday_valid_input(date, day, expected_result):
    """
    Test for get_next_same_weekday function with valid input.
    """
    assert get_next_same_weekday(date, day) == expected_result


@pytest.mark.parametrize(
    "date, day",
    [
        (datetime.datetime(2024, 4, 1), 0),  # Invalid day
        (datetime.datetime(2024, 4, 1), 8),  # Invalid day
    ],
)
def test_get_next_same_weekday_invalid_input(date, day):
    """
    Test for get_next_same_weekday function with invalid input.
    """
    with pytest.raises(ValueError):
        get_next_same_weekday(date, day)
