# 📄 Dosya Yolu: /turkuazvm/guest/android-image/aosp-overlay/device/turkuazvm/cuttlefish/AndroidProducts.mk
# 📌 Amac: Turkuaz Android Cuttlefish product makefile ve lunch targetini AOSP build sistemine kaydeder
# 📌 Modul - Make
# Version: 0.12.4
# Aciklama: x86_64 phone productunu aosp_current userdebug build profiliyle kullanilabilir yapar
# Bagimli Oldugu Katman: Config | Tool

PRODUCT_MAKEFILES := \
    $(LOCAL_DIR)/turkuazvm_cf_x86_64_phone.mk

COMMON_LUNCH_CHOICES := \
    turkuazvm_cf_x86_64_phone-aosp_current-userdebug
