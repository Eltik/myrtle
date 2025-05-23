# -*- coding: utf-8 -*-
# Copyright (c) 2022-2025, Harry Huang
# @ BSD 3-Clause License
import json
import os
import os.path as osp

from .Logger import Logger


def get_cpu_count():
    cpu = os.cpu_count()
    return 1 if cpu is None or cpu <= 0 else cpu


class PerformanceLevel:
    """Enumeration class for performance level."""

    MINIMAL = 0
    LOW = 1
    STANDARD = 2
    HIGH = 3

    __CPU = get_cpu_count()
    __MAP = {
        MINIMAL: 1,
        LOW: max(2, __CPU // 2),
        STANDARD: max(4, __CPU),
        HIGH: max(8, __CPU * 2),
    }

    @staticmethod
    def get_thread_limit(performance_level: int):
        """Gets the maximum thread count according to the given performance level."""
        return PerformanceLevel.__MAP.get(
            performance_level, PerformanceLevel.__MAP[PerformanceLevel.STANDARD]
        )


class Config:
    """Configuration class for ArkUnpacker."""

    __instance = None
    __config_path = "ArkUnpackerConfig.json"
    __file_encoding = "UTF-8"
    __default_config = {
        "log_file": "ArkUnpackerLogs.log",
        "log_level": Logger.LV_INFO,
        "performance_level": PerformanceLevel.STANDARD,
    }

    def __init__(self):
        """Not recommended to use. Please use the static methods."""
        self._config = {}

    def _get(self, key: str):
        return self._config[key]

    def _read_config(self):
        if osp.isfile(Config.__config_path):
            try:
                loaded_config = json.load(
                    open(Config.__config_path, "r", encoding=Config.__file_encoding)
                )
                if isinstance(loaded_config, dict):
                    for k in Config.__default_config.keys():
                        default_val = Config.__default_config[k]
                        self._config[k] = (
                            loaded_config[k]
                            if isinstance(loaded_config.get(k, None), type(default_val))
                            else default_val
                        )
                Logger.set_instance(self.get("log_file"), self.get("log_level"))
                Logger.set_level(self.get("log_level"))
                Logger.info("Config: Applied config.")
            except Exception as arg:
                self._config = Config.__default_config
                Logger.set_instance(self.get("log_file"), self.get("log_level"))
                Logger.set_level(self.get("log_level"))
                Logger.error(
                    f"Config: Failed to parsing config, now using default config, cause: {arg}"
                )
        else:
            self._config = Config.__default_config
            Logger.set_instance(self.get("log_file"), self.get("log_level"))
            Logger.set_level(self.get("log_level"))
            Logger.info("Config: Applied default config.")
            self.save_config()

    def _save_config(self):
        try:
            json.dump(
                self._config,
                open(self.__config_path, "w", encoding=Config.__file_encoding),
                indent=4,
                ensure_ascii=False,
            )
            Logger.info("Config: Saved config.")
        except Exception as arg:
            Logger.error(f"Config: Failed to save config, cause: {arg}")

    @staticmethod
    def _get_instance():
        if not Config.__instance:
            Config.__instance = Config()
            Config.__instance._read_config()
        return Config.__instance

    @staticmethod
    def get(key):
        """Gets the specified config field.

        :param key: The JSON key to the field;
        :returns: The value of the field, `None` if the key doesn't exist;
        :rtype: Any;
        """
        return Config._get_instance()._get(key)

    @staticmethod
    def read_config():
        """Reads the config from file, aka. deserialize the config.
        The default config will be used if the config file doesn't exist or an error occurs.
        The logging level of `Logger` class will be updated according to the config.
        """
        return Config._get_instance()._read_config()

    @staticmethod
    def save_config():
        """Saves the config to file, aka. serialize the config."""
        return Config._get_instance()._save_config()
