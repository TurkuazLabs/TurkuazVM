# 📄 Dosya Yolu: /turkuazvm/guest/android-image/aosp-overlay/device/turkuazvm/cuttlefish/turkuazvm_cf_x86_64_phone.mk
# 📌 Amac: TurkuazVM Android gaming image product ozellestirmesini tanimlar
# 📌 Modul - Make
# Version: 0.12.4
# Aciklama: AOSP Cuttlefish x86_64-only phone productunu devralir ve Turkuaz Input Agent'i product image'a ekler
# Bagimli Oldugu Katman: Config | Service | Tool

$(call inherit-product, device/google/cuttlefish/vsoc_x86_64_only/phone/aosp_cf.mk)

PRODUCT_NAME := turkuazvm_cf_x86_64_phone
PRODUCT_DEVICE := vsoc_x86_64
PRODUCT_BRAND := TurkuazVM
PRODUCT_MODEL := Turkuaz Android Gaming Device
PRODUCT_MANUFACTURER := TurkuazVM

PRODUCT_PACKAGES += \
    TurkuazInputAgent

PRODUCT_PRODUCT_PROPERTIES += \
    ro.turkuazvm.image=1 \
    ro.turkuazvm.gaming_input_agent=1 \
    ro.turkuazvm.arm_translation=0
