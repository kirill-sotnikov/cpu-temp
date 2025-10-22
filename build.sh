#!/bin/bash

# Скрипт для создания .app бандла и .dmg образа для macOS
# Использование: ./build_app.sh <путь_к_бинарнику> <имя_приложения> <bundle_id> [путь_к_иконке]

set -e  # Остановка при ошибке

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Проверка аргументов
if [ $# -lt 3 ]; then
    echo -e "${RED}Ошибка: недостаточно аргументов${NC}"
    echo "Использование: $0 <путь_к_бинарнику> <имя_приложения> <bundle_id> [путь_к_иконке]"
    echo "Пример: $0 ./myapp MyApp com.company.myapp ./icon.icns"
    exit 1
fi

BINARY_PATH="$1"
APP_NAME="$2"
BUNDLE_ID="$3"
ICON_PATH="${4:-}"

# Проверка существования бинарного файла
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Ошибка: файл '$BINARY_PATH' не найден${NC}"
    exit 1
fi

# Проверка иконки, если указана
if [ -n "$ICON_PATH" ] && [ ! -f "$ICON_PATH" ]; then
    echo -e "${YELLOW}Предупреждение: файл иконки '$ICON_PATH' не найден, продолжаем без иконки${NC}"
    ICON_PATH=""
fi

APP_BUNDLE="${APP_NAME}.app"
DMG_NAME="${APP_NAME}.dmg"
DMG_TEMP="dmg_temp"

echo -e "${GREEN}=== Создание macOS приложения ===${NC}"
echo "Имя приложения: $APP_NAME"
echo "Bundle ID: $BUNDLE_ID"
echo "Бинарный файл: $BINARY_PATH"

# Удаление старых файлов
if [ -d "$APP_BUNDLE" ]; then
    echo "Удаление существующего $APP_BUNDLE..."
    rm -rf "$APP_BUNDLE"
fi

if [ -f "$DMG_NAME" ]; then
    echo "Удаление существующего $DMG_NAME..."
    rm -f "$DMG_NAME"
fi

# Создание структуры .app бандла
echo -e "${GREEN}Шаг 1: Создание структуры директорий${NC}"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Копирование бинарного файла
echo -e "${GREEN}Шаг 2: Копирование бинарного файла${NC}"
cp "$BINARY_PATH" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# Создание Info.plist
echo -e "${GREEN}Шаг 3: Создание Info.plist${NC}"
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
EOF

# Добавление иконки, если она есть
if [ -n "$ICON_PATH" ]; then
    echo -e "${GREEN}Шаг 4: Добавление иконки${NC}"
    cp "$ICON_PATH" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
    cat >> "$APP_BUNDLE/Contents/Info.plist" << EOF
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
EOF
else
    echo -e "${YELLOW}Шаг 4: Пропущен (иконка не указана)${NC}"
fi

# Закрытие Info.plist
cat >> "$APP_BUNDLE/Contents/Info.plist" << EOF
</dict>
</plist>
EOF

# Подпись приложения (самоподписанная)
echo -e "${GREEN}Шаг 5: Подпись приложения${NC}"
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || {
    echo -e "${YELLOW}Предупреждение: не удалось подписать приложение${NC}"
}

echo -e "${GREEN}✓ Приложение $APP_BUNDLE успешно создано!${NC}"

# Создание DMG образа
echo -e "${GREEN}=== Создание DMG образа ===${NC}"

# Создание временной директории для DMG
rm -rf "$DMG_TEMP"
mkdir -p "$DMG_TEMP"

# Копирование .app в временную директорию
echo -e "${GREEN}Шаг 6: Подготовка содержимого DMG${NC}"
cp -r "$APP_BUNDLE" "$DMG_TEMP/"

# Создание симлинка на Applications
ln -s /Applications "$DMG_TEMP/Applications"

# Создание DMG
echo -e "${GREEN}Шаг 7: Создание DMG образа${NC}"
hdiutil create -volname "$APP_NAME" \
    -srcfolder "$DMG_TEMP" \
    -ov -format UDZO \
    "$DMG_NAME"

# Очистка
echo -e "${GREEN}Шаг 8: Очистка временных файлов${NC}"
rm -rf "$DMG_TEMP"

# Итоговая информация
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}✓ Сборка завершена успешно!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Созданные файлы:"
echo "  • $APP_BUNDLE - приложение macOS"
echo "  • $DMG_NAME - установочный образ"
echo ""
echo "Размеры файлов:"
du -h "$APP_BUNDLE" | tail -1
du -h "$DMG_NAME"
echo ""
echo "Для запуска приложения: open $APP_BUNDLE"
echo "Для открытия DMG: open $DMG_NAME"
echo ""
