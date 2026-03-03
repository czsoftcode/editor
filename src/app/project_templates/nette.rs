use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path, version: &str, php_version: &str) -> Result<(), String> {
    let composer_json = if version == "3.2" {
        format!(
            r#"{{
    "name": "nette/{0}",
    "description": "Nette Web Project 2026",
    "type": "project",
    "license": ["MIT", "BSD-3-Clause"],
    "require": {{
        "php": ">= {1}",
        "nette/application": "^3.2.3",
        "nette/bootstrap": "^3.2.6",
        "nette/caching": "^3.2",
        "nette/database": "^3.2",
        "nette/di": "^3.2",
        "nette/forms": "^3.2",
        "nette/http": "^3.3",
        "nette/mail": "^4.0",
        "nette/robot-loader": "^4.0",
        "nette/security": "^3.2",
        "nette/utils": "^4.0",
        "latte/latte": "^3.1",
        "tracy/tracy": "^2.11"
    }},
    "require-dev": {{
        "nette/tester": "^2.5",
        "phpstan/phpstan-nette": "^1.3"
    }},
    "autoload": {{
        "psr-4": {{
            "App\\": "app"
        }}
    }},
    "config": {{
        "allow-plugins": {{
            "phpstan/extension-installer": true
        }}
    }}
}}"#,
            name, php_version
        )
    } else {
        format!(
            r#"{{
    "name": "nette/{0}",
    "description": "Nette Web Project 3.0 LTS",
    "type": "project",
    "license": ["MIT", "BSD-3-Clause"],
    "require": {{
        "php": ">= {1}",
        "nette/application": "^3.0.7",
        "nette/bootstrap": "^3.0.2",
        "nette/caching": "^3.0.2",
        "nette/database": "^3.0.7",
        "nette/di": "^3.0.5",
        "nette/forms": "^3.0.3",
        "nette/http": "^3.0.6",
        "nette/mail": "^3.0.1",
        "nette/robot-loader": "^3.3",
        "nette/security": "^3.0.3",
        "nette/utils": "^3.1",
        "latte/latte": "^2.8",
        "tracy/tracy": "^2.8"
    }},
    "require-dev": {{
        "nette/tester": "^2.3",
        "phpstan/phpstan-nette": "^0.12"
    }},
    "autoload": {{
        "psr-4": {{
            "App\\": "app"
        }}
    }}
}}"#,
            name, php_version
        )
    };

    let bootstrap_php = r#"<?php
declare(strict_types=1);
namespace App;
use Nette\Bootstrap\Configurator;

class Bootstrap
{
    public static function boot(): Configurator
    {
        $configurator = new Configurator;
        $rootDir = dirname(__DIR__);
        $configurator->enableTracy($rootDir . '/log');
        $configurator->setTempDirectory($rootDir . '/temp');
        $configurator->createRobotLoader()->addDirectory(__DIR__)->register();
        $configurator->addConfig($rootDir . '/config/common.neon');
        return $configurator;
    }
}
"#;

    fs::create_dir_all(path.join("app")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("config")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("log")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("temp")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("www")).map_err(|e| e.to_string())?;

    fs::write(path.join("composer.json"), composer_json).map_err(|e| e.to_string())?;
    fs::write(path.join("app/Bootstrap.php"), bootstrap_php).map_err(|e| e.to_string())?;
    fs::write(path.join("www/index.php"), "<?php\ndeclare(strict_types=1);\nrequire __DIR__ . '/../vendor/autoload.php';\nApp\\Bootstrap::boot()->createContainer()->getByType(Nette\\Application\\Application::class)->run();\n").map_err(|e| e.to_string())?;
    fs::write(path.join(".gitignore"), "/vendor/\n/temp/\n/log/\n").map_err(|e| e.to_string())?;

    Ok(())
}
