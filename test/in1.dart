// ignore_for_file: public_member_api_docs, sort_constructors_first

part 'preview.copy.gen.dart';

// +mk:copyWithMixin
class UiInsurancePreviewSimpleDataModel {
  final String? title;
  final Set<String?> cta;
  final List<String>? image;
  final List<String> image2;

  const UiInsurancePreviewSimpleDataModel({
    required this.title,
    required this.cta,
    required this.image,
    required this.image2,
  });
}

// +mk:copyWith
class UiInsurancePreviewDataModel {
  final String? avatar;
  final String tagType;

  const UiInsurancePreviewDataModel({
    required this.avatar,
    required this.tagType,
  });
}

// +mk:copyWithNullableValue
class UiInsurancePreviewDataModel2 {
  final String? avatar;
  final String tagType;

  const UiInsurancePreviewDataModel2({
    required this.avatar,
    required this.tagType,
  });
}