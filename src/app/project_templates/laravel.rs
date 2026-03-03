use std::fs;
use std::path::Path;

pub(crate) fn generate(name: &str, path: &Path) -> Result<(), String> {
    let composer_json = format!(
        r#"{{
    "name": "laravel/{0}",
    "type": "project",
    "description": "The Laravel Framework.",
    "keywords": ["framework", "laravel"],
    "license": "MIT",
    "require": {{
        "php": "^8.2",
        "laravel/framework": "^12.0",
        "laravel/tinker": "^2.10",
        "laravel/sanctum": "^4.0"
    }},
    "require-dev": {{
        "fakerphp/faker": "^1.23",
        "laravel/sail": "^1.28",
        "mockery/mockery": "^1.6",
        "nunomaduro/collision": "^8.1",
        "phpunit/phpunit": "^11.0",
        "spatie/laravel-ignition": "^2.4"
    }},
    "autoload": {{
        "psr-4": {{
            "App\\": "app/",
            "Database\\Factories\\": "database/factories/",
            "Database\\Seeders\\": "database/seeders/"
        }}
    }},
    "config": {{
        "optimize-autoloader": true,
        "preferred-install": "dist",
        "sort-packages": true,
        "allow-plugins": {{
            "pestphp/pest-plugin": true,
            "php-http/discovery": true
        }}
    }},
    "minimum-stability": "stable",
    "prefer-stable": true
}}"#,
        name
    );

    let artisan_bin = r#"#!/usr/bin/env php
<?php
define('LARAVEL_START', microtime(true));
require __DIR__.'/vendor/autoload.php';
$status = (require_once __DIR__.'/bootstrap/app.php')
    ->handleCommand(new Symfony\Component\Console\Input\ArgvInput);
exit($status);
"#;

    let index_php = r#"<?php
use Illuminate\Http\Request;
define('LARAVEL_START', microtime(true));
require __DIR__.'/../vendor/autoload.php';
(require_once __DIR__.'/../bootstrap/app.php')
    ->handleRequest(Request::capture());
"#;

    fs::create_dir_all(path.join("app")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("bootstrap")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("public")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("routes")).map_err(|e| e.to_string())?;
    fs::create_dir_all(path.join("storage")).map_err(|e| e.to_string())?;

    fs::write(path.join("composer.json"), composer_json).map_err(|e| e.to_string())?;
    fs::write(path.join("artisan"), artisan_bin).map_err(|e| e.to_string())?;
    fs::write(path.join("public/index.php"), index_php).map_err(|e| e.to_string())?;
    fs::write(path.join(".env.example"), "APP_NAME=Laravel\nAPP_ENV=local\n").map_err(|e| e.to_string())?;
    fs::write(path.join(".gitignore"), "/vendor/\n.env\n").map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(path.join("artisan")) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(path.join("artisan"), perms);
        }
    }

    Ok(())
}
